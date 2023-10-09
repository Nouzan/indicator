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
///
/// ## Note
/// - When using with the `generate_out` flag, the output type will be converted to the same
/// as `generate_out_with_data`, i.e. `(OutTy, Option<DataTy>)` where `DataTy = ()`
/// is this case. So that it can be used in `insert` method (but not the `insert_env` method).
/// - But when using with the `generate_data` flag, the output type is kept as `Option<DataTy>`,
/// so that it can be used in `insert_data` method.
///
/// ## Example
/// The following example is a basic usage of `#[operator]` attribute.
/// We use `#[operator]` to create a `RefOperator` that just add two to the input.
/// ```rust
/// use indicator::prelude::*;
/// use num::Num;
///
/// struct AddTwo<T>(T);
///
/// /// An operator that just add two to the input.
/// #[operator(input = T)]
/// fn add_two<T>(In(value): In<&T>) -> AddTwo<T>
/// where
///     T: Num + Clone,
///     T: Send + Sync + 'static,
/// {
///     let two = T::one() + T::one();
///     AddTwo(value.clone() + two)
/// }
///
/// let op = input::<i32>().insert_env(add_two).finish();
/// ```
///
/// We notice that the above example requires us to define a struct `AddTwo` to wrap the output.
/// The same operator can also be created with the following code but in a more generic way.
/// ```rust
/// use indicator::prelude::*;
/// use num::Num;
///
/// /// An operator that just add two to the input.
/// #[operator(input = I, generate_out)]
/// fn add_two<T>(#[input] value: &T) -> T
/// where
///    T: Num + Clone,
/// {
///    let two = T::one() + T::one();
///   value.clone() + two
/// }
///
/// let op = input::<i32>().insert(add_two::<_, _, i32>).finish();
/// ```
/// Here we use the `#[input]` attribute and the `generate_out` to achieve our goal.
/// - The `#[input]` attribute is used to mark the input parameter.
/// It will replace the `value: &T` parameter with `In(value): In<&I>` in the generated code,
/// and introduce a bound `I: Borrow<T>` so that it can be converted back to `&T` before calling.
/// - The `generate_out` flag in the `#[operator]` attribute is used to generate a generic output type.
/// Like what the `#[input]` attribute does, it will introduce a bound `OutTy: From<T>`
/// (here the `OutTy` is the generated output type) so that the output can be converted from `T`.
///
/// Also note that the operator generated with `generate_out` flag is meant to be used with `insert` method,
/// because the output type is wrapped in a tuple `(OutTy, Option<DataTy>)` where `DataTy = ()` is this case.
/// If you want to costomize the `DataTy`, you can use the `generate_out_with_data` flag instead.
#[proc_macro_attribute]
pub fn operator(args: TokenStream, input: TokenStream) -> TokenStream {
    match self::operator::generate_operator(args, input) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
