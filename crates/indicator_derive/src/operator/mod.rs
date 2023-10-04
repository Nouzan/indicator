use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use self::{args::OperatorArgs, operator_fn::OperatorFn};
use super::indicator;

/// Arguments for generating operator.
mod args;

/// Operator Fn.
mod operator_fn;

mod utils;

/// Extractor.
mod extractor;

pub(super) fn generate_operator(
    args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream2> {
    let args = syn::parse::<OperatorArgs>(args)?;
    let expanded = OperatorFn::parse_with(input.clone().into(), &args)?.expand();
    Ok(expanded)
}
