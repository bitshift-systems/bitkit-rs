use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod expand;

#[proc_macro_derive(Error, attributes(error, from))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input).into()
}
