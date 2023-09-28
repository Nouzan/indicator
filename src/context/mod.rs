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

/// `Value`-operator.
/// Alias for `Operator<Value<T>>`.
pub trait ContextOperator<T>: Operator<Value<T>> {
    /// The output type.
    /// Just an alias for `Self::Output`.
    type Out: IntoValue;

    /// Call the `next` method of the operator.
    fn call(&mut self, input: Value<T>) -> Self::Out;

    /// Apply a layer.
    fn with<L>(self, layer: L) -> L::Output
    where
        L: Layer<T, Self>,
        Self: Sized,
    {
        layer.layer(self)
    }
}

impl<T, P> ContextOperator<T> for P
where
    P: Operator<Value<T>>,
    P::Output: IntoValue,
{
    type Out = P::Output;

    #[inline]
    fn call(&mut self, input: Value<T>) -> Self::Out {
        self.next(input)
    }
}
