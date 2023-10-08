use crate::context::{Context, ContextOperator, Value};

use super::Layer;

/// Layer that used to inspect the context.
pub struct Then<F>(pub F);

impl<T, U, P, Builder, F> Layer<T, P> for Then<Builder>
where
    P: ContextOperator<T>,
    Builder: Fn() -> F,
    F: FnMut(P::Out, &Context) -> U + Clone,
{
    type Operator = ThenOperator<P, F>;
    type Out = U;

    #[inline]
    fn layer(&self, operator: P) -> Self::Operator {
        ThenOperator {
            inner: operator,
            then: (self.0)(),
        }
    }
}

/// Operator that used to inspect the context.
pub struct ThenOperator<P, F> {
    inner: P,
    then: F,
}

impl<T, U, P, F> ContextOperator<T> for ThenOperator<P, F>
where
    P: ContextOperator<T>,
    F: FnMut(P::Out, &Context) -> U,
{
    type Out = U;

    #[inline]
    fn next(&mut self, input: Value<T>) -> Value<Self::Out> {
        let Value { value, context } = self.inner.next(input);
        let out = (self.then)(value, &context);
        Value {
            value: out,
            context,
        }
    }
}
