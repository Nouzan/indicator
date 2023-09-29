use crate::context::ValueRef;

use super::FromValueRef;

/// Extract the input from [`ValueRef`].
pub struct In<T>(pub T);

impl<'a, T> FromValueRef<'a, T> for In<&'a T> {
    #[inline]
    fn from_value_ref(value: &ValueRef<'a, T>) -> Self {
        In(value.value)
    }
}
