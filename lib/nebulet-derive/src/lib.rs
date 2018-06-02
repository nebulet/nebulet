#![feature(proc_macro)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(KernelRef)]
pub fn kernel_ref(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_kernel_ref(&ast).into()
}

#[proc_macro_attribute]
pub fn nebulet_abi(_args: TokenStream, input: TokenStream) -> TokenStream {
    let fn_item = syn::parse(input).unwrap();

    wrap_nebulet_abi(fn_item).into()
}

fn impl_kernel_ref(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = &ast.ident;
    if let syn::Data::Struct(_) = ast.data {
        quote! {
            impl KernelRef for #name {}
        }
    } else {
        panic!("#[derive(KernelRef)] is only defined for structs.");
    }
}

fn wrap_nebulet_abi(mut fn_item: syn::ItemFn) -> proc_macro2::TokenStream {
    let outer_func = fn_item.clone();
    let outer_ident = outer_func.ident;
    let mut outer_inputs = outer_func.decl.inputs;
    outer_inputs.pop();
    outer_inputs.push(syn::parse::<syn::FnArg>(quote!(vmctx: *const ()).into()).unwrap());

    let ident_span = fn_item.ident.span();
    fn_item.ident = syn::Ident::new("inner", ident_span);
    fn_item.vis = syn::Visibility::Inherited;
    let mut inner_inputs = outer_inputs
        .iter()
        .filter_map(|fnarg| {
            if let syn::FnArg::Captured(arg) = fnarg {
                Some(arg.pat.clone())
            } else {
                None
            }
        })
        .collect::<syn::punctuated::Punctuated<syn::Pat, syn::token::Comma>>();
    inner_inputs.pop();
    inner_inputs.push(syn::parse(quote!(process).into()).unwrap());

    // TODO: More generic handling of return type.
    if fn_item.decl.output == syn::ReturnType::Default {
        quote! {
            pub extern fn #outer_ident(#outer_inputs) {
                #fn_item

                use wasm::instance::VmCtx;
                let vmctx = unsafe { &*(vmctx as *const VmCtx) };

                let process = &vmctx.process;
                inner(#inner_inputs)
            }
        }
    }
    else {
        quote! {
            pub extern fn #outer_ident(#outer_inputs) -> u64 {
                #fn_item

                use wasm::instance::VmCtx;
                let vmctx = unsafe { &*(vmctx as *const VmCtx) };

                let process = &vmctx.process;
                let res = inner(#inner_inputs);
                Error::mux(res)
            }
        }
    }
}
