extern crate serde_osc;
extern crate proc_macro;

#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(NumFields)]
pub fn num_fields(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    // Parse the string representation into a syntax tree
    let ast = syn::parse_derive_input(&source).unwrap();

    // Build the output
    let expanded = impl_hello_world(&ast);

    // Return the generated impl as a TokenStream
    expanded.parse().unwrap()
}

fn impl_hello_world(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        impl NumFields for #name {
            fn num_fields() -> usize {
                1
            }
        }
    }
}
