use core::marker::PhantomData;

use crate::context::ContextOperator;

use super::Layer;

/// Stack of layers.
#[derive(Debug, Clone, Copy)]
pub struct Stack<Inner, Outer>(pub(super) Inner, pub(super) Outer);

impl<T, P, Inner, Outer> Layer<T, P> for Stack<Inner, Outer>
where
    P: ContextOperator<T>,
    Inner: Layer<T, P>,
    Outer: Layer<T, Inner::Operator>,
{
    type Operator = Outer::Operator;
    type Out = <Self::Operator as ContextOperator<T>>::Out;

    #[inline]
    fn layer(&self, operator: P) -> Self::Operator {
        self.1.layer(self.0.layer(operator))
    }
}

/// Identity Layer.
#[derive(Debug, Clone, Copy)]
pub struct Identity<T, P>(PhantomData<fn() -> (T, P)>);

impl<T, P> Layer<T, P> for Identity<T, P>
where
    P: ContextOperator<T>,
{
    type Operator = P;
    type Out = P::Out;

    /// [`Identity`] is a layer that maps the operator to itself.
    #[inline]
    fn layer(&self, operator: P) -> Self::Operator {
        operator
    }
}

/// Create an identity layer.
pub fn id_layer<T, P>() -> Identity<T, P> {
    Identity(PhantomData)
}
