mod anymap;

/// Value with context.
pub mod value;

/// Convert an `In`-operator to another `In`-operator.
pub mod layer;

use crate::Operator;

pub use self::{
    anymap::Context,
    layer::Layer,
    value::{input, Input, IntoValue, Value},
};

/// Operator that takes a `Value` as input and returns a `Value` as output.
/// And can be converted to an operator without the `Value` wrapper.
pub trait ContextOperator<T> {
    /// The output type.
    /// Just an alias for `Self::Output`.
    type Output: IntoValue;

    /// Apply the operator.
    fn next(&mut self, input: Value<T>) -> Self::Output;

    /// Apply a layer.
    fn with<L>(self, layer: L) -> L::Output
    where
        L: Layer<T, Self>,
        Self: Sized,
    {
        layer.layer(self)
    }

    /// Build into an operator without the `Value` wrapper.
    fn finish(self) -> ContextedOperator<Self>
    where
        Self: Sized,
    {
        ContextedOperator(self)
    }
}

impl<T, P> ContextOperator<T> for P
where
    P: Operator<Value<T>>,
    P::Output: IntoValue,
{
    type Output = P::Output;

    #[inline]
    fn next(&mut self, input: Value<T>) -> Self::Output {
        self.next(input)
    }
}

/// Context Operator.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContextedOperator<P>(P);

impl<T, P> Operator<T> for ContextedOperator<P>
where
    P: ContextOperator<T>,
{
    type Output = <<P as ContextOperator<T>>::Output as IntoValue>::Inner;

    #[inline]
    fn next(&mut self, input: T) -> Self::Output {
        self.0.next(Value::new(input)).into_value().into_inner()
    }
}
