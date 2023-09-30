use crate::context::ValueRef;

use super::FromValueRef;

/// Extract from the context.
pub struct Env<T>(pub T);

impl<'a, I, T> FromValueRef<'a, I> for Env<&'a T>
where
    T: Send + Sync + 'static,
{
    #[inline]
    fn from_value_ref(value: &ValueRef<'a, I>) -> Self {
        Self(value.context.env().get::<T>().unwrap_or_else(|| {
            panic!("`{}` not found in the context", core::any::type_name::<T>())
        }))
    }
}
