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
