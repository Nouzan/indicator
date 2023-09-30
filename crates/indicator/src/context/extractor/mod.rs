use super::ValueRef;

/// Input extractor.
pub mod input;

/// Extract from the context.
pub mod env;

/// Extract from previous context.
pub mod prev;

/// Extract from the data context.
pub mod data;

pub use self::{data::Data, env::Env, input::In, prev::Prev};

/// Type that can extract from [`ValueRef`].
pub trait FromValueRef<'a, T> {
    /// Extrace from [`ValueRef`].
    fn from_value_ref(value: &ValueRef<'a, T>) -> Self;
}
