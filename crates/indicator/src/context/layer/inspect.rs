use crate::context::{Context, ContextOperator, Value, ValueRef};

use super::Layer;

/// Layer that used to inspect the context.
pub struct Inspect<F>(pub F);

impl<T, P, F> Layer<T, P> for Inspect<F>
where
    P: ContextOperator<T>,
    F: Fn(&T, &Context) + Clone,
{
    type Operator = InspectOperator<P, F>;
    type Out = P::Out;

    #[inline]
    fn layer(&self, operator: P) -> Self::Operator {
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
    F: Fn(&T, &Context),
{
    type Out = P::Out;

    #[inline]
    fn next(&mut self, input: Value<T>) -> Value<Self::Out> {
        let ValueRef { value, context } = input.as_ref();
        (self.inspect)(value, context);
        self.inner.next(input)
    }
}
