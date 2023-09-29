use super::ValueRef;

/// Input extractor.
pub mod input;

pub use self::input::In;

/// Type that can extract from [`ValueRef`].
pub trait FromValueRef<'a, T> {
    /// Extrace from [`ValueRef`].
    fn from_value_ref(value: &ValueRef<'a, T>) -> Self;
}
