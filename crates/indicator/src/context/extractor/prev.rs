use crate::context::{layer::cache::Previous, ValueRef};

use super::FromValueRef;

/// Extract from previous context.
pub struct Prev<T>(pub Option<T>);

impl<'a, I, T> FromValueRef<'a, I> for Prev<&'a T>
where
    T: Send + Sync + 'static,
{
    #[inline]
    fn from_value_ref(value: &ValueRef<'a, I>) -> Self {
        Self(value.context.get::<Previous>().and_then(|prev| prev.get()))
    }
}
