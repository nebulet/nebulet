#![feature(proc_macro_span, proc_macro_diagnostic, iterator_find_map,)]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

// enum DispatchType {
//     Solo,
//     Peer,
// }

// #[proc_macro_derive(Dispatcher, attributes(DispatcherType))]
// pub fn dispatcher(input: TokenStream) -> TokenStream {
//     let ast = syn::parse(input).unwrap();

//     impl_dispatcher(&ast).into()
// }

// fn impl_dispatcher(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
//     let dispatch_type = ast.attrs.clone().iter().find_map(|attr| {
//         if let Some(syn::Meta::NameValue(nv)) = attr.interpret_meta() {
//             if nv.ident.to_string() == "dispatch_type" {
//                 if let syn::Lit::Str(s) = nv.lit {
//                     use quote::ToTokens;
//                     match s.into_token_stream().to_string().as_str() {
//                         "\"solo\"" => return Some(DispatchType::Solo),
//                         "\"peer\"" => return Some(DispatchType::Peer),
//                         _ => return None,
//                     }
//                 }
//             }
//         }
//         None
//     }).expect("Incorrect attribute for #[derive(Dispatcher)]");

//     let name = &ast.ident;
//     if let syn::Data::Struct(_) = ast.data {
//         let context_associated_type: syn::TraitItemType =  match dispatch_type {
//             DispatchType::Solo => {
//                 quote! {
//                     type Context = SoloContext;
//                 }
//             },
//             DispatchType::Peer => {
//                 quote! {
//                     type Context = PeerContext;
//                 }
//             },
//         };

//         let dispatch: syn::Ident = match dispatch_type {
//             DispatchType::Solo => quote! { SoloDispatcher },
//             DispatchType::Peer => quote! { PeerDispatcher },
//         };

//         quote! {
//             impl Dispatcher for #name {
//                 fn allowed_user_signals() -> Signal {
//                     #dispatch::allowed_user_signals
//                 }

//                 fn signal(&mut self, ctx: &mut Context, set_signals: Signal, clear_signals: Signal, peer: bool) -> Result<()>;

//                 fn update_state(&mut self, ctx: &mut Context, set_signals: Signal, clear_signals: Signal) {
//                     debug_assert!(Self::allows_observers());

//                     let previous_signals = ctx.signals;
//                     ctx.signals.remove(clear_signals);
//                     ctx.signals.insert(set_signals);

//                     if previous_signals == ctx.signals {
//                         return;
//                     }

//                     ctx.observers.retain(|observer| {
//                         match observer.on_state_change(ctx.signals) {
//                             ObserverResult::Keep => true,
//                             ObserverResult::Remove => {
//                                 observer.on_removal();
//                                 false
//                             },
//                         }
//                     });
//                 }

//                 const fn allows_observers() -> bool { false }

//                 fn get_name(&self) -> Option<&str> { None }
//                 fn set_name(&mut self) -> Result<()> { Err(Error::NOT_SUPPORTED) }
//             }
//         }
//     } else {
//         panic!("#[derive(Dispatcher)] is only defined for structs.");
//     }
// }

#[proc_macro_attribute]
pub fn nebulet_abi(_args: TokenStream, input: TokenStream) -> TokenStream {
    let fn_item = syn::parse(input).unwrap();

    wrap_nebulet_abi(fn_item).into()
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
        }).collect::<syn::punctuated::Punctuated<syn::Pat, syn::token::Comma>>();
    inner_inputs.pop();
    inner_inputs.push(syn::parse(quote!(user_data).into()).unwrap());

    // TODO: More generic handling of return type.
    if fn_item.decl.output == syn::ReturnType::Default {
        quote! {
            pub extern fn #outer_ident(#outer_inputs) {
                #[inline]
                #fn_item

                use wasm::instance::VmCtx;
                let vmctx = unsafe { &*(vmctx as *const VmCtx) };
                let user_data = &vmctx.data().user_data;

                inner(#inner_inputs);
            }
        }
    } else {
        quote! {
            pub extern fn #outer_ident(#outer_inputs) -> u64 {
                #[inline]
                #fn_item

                use wasm::instance::VmCtx;
                let vmctx = unsafe { &*(vmctx as *const VmCtx) };
                let user_data = &vmctx.data().user_data;

                let res = inner(#inner_inputs);
                Error::mux(res)
            }
        }
    }
}
