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
        Self(
            value
                .context
                .env()
                .get::<Previous>()
                .expect(
                    "`Previous` not found in the context. Perhaps you forgot to add `Cache` layer?",
                )
                .get::<T>(),
        )
    }
}
