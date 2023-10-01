use crate::context::ValueRef;

use super::FromValueRef;

/// Extract from the data context.
pub struct Data<T>(pub T);

impl<'a, I, T> FromValueRef<'a, I> for Data<&'a T>
where
    T: Send + Sync + 'static,
{
    #[inline]
    fn from_value_ref(value: &ValueRef<'a, I>) -> Self {
        Self(value.context.data().get::<T>().unwrap_or_else(|| {
            panic!(
                "`{}` not found in the data context",
                core::any::type_name::<T>()
            )
        }))
    }
}

impl<'a, I, T> FromValueRef<'a, I> for Data<Option<&'a T>>
where
    T: Send + Sync + 'static,
{
    #[inline]
    fn from_value_ref(value: &ValueRef<'a, I>) -> Self {
        Self(value.context.data().get::<T>())
    }
}
