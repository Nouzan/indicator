use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::Ident;

mod operator;

fn indicator() -> TokenStream2 {
    let found_crate = crate_name("indicator").expect("my-crate is present in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident )
        }
    }
}

/// Create a `RefOperator` from a function.
#[proc_macro_attribute]
pub fn operator(args: TokenStream, input: TokenStream) -> TokenStream {
    match self::operator::generate_operator(args, input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
