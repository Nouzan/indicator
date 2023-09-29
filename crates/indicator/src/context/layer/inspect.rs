use crate::context::{ContextOperator, Value, ValueRef};

use super::Layer;

/// Layer that used to inspect the context.
pub struct Inspect<F>(pub F);

impl<T, P, F> Layer<T, P> for Inspect<F>
where
    P: ContextOperator<T>,
    F: Fn(ValueRef<'_, T>) + Clone,
{
    type Output = InspectOperator<P, F>;

    #[inline]
    fn layer(&self, operator: P) -> Self::Output {
        InspectOperator {
            inner: operator,
            inspect: self.0.clone(),
        }
    }
}

/// Operator that used to inspect the context.
pub struct InspectOperator<P, F> {
    inner: P,
    inspect: F,
}

impl<T, P, F> ContextOperator<T> for InspectOperator<P, F>
where
    P: ContextOperator<T>,
    F: Fn(ValueRef<'_, T>),
{
    type Output = P::Output;

    #[inline]
    fn next(&mut self, input: Value<T>) -> Self::Output {
        (self.inspect)(input.as_ref());
        self.inner.next(input)
    }
}
