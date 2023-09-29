use super::ValueRef;

/// Input extractor.
pub mod input;

/// Extract from the context.
pub mod env;

/// Extract from previous context.
pub mod prev;

pub use self::{env::Env, input::In, prev::Prev};

/// Type that can extract from [`ValueRef`].
pub trait FromValueRef<'a, T> {
    /// Extrace from [`ValueRef`].
    fn from_value_ref(value: &ValueRef<'a, T>) -> Self;
}
