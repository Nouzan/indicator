use self::stack::Stack;

use super::ContextOperator;

/// Layer that caches the final context,
/// and provides it to the next evaluation.
pub mod cache;

/// Layer that inserts a value into the context.
pub mod insert;

/// Layer that used to inspect the context.
pub mod inspect;

/// Layer for manipulating the data context.
pub mod data;

/// Stack of layers.
pub mod stack;

/// Layer.
/// Convert an [`ContextOperator`] to another [`ContextOperator`]
pub trait Layer<In, P>
where
    P: ContextOperator<In>,
{
    /// The output operator.
    type Operator: ContextOperator<In, Out = Self::Out>;

    /// The output type.
    type Out;

    /// Convert an `In`-operator to another `In`-operator.
    fn layer(&self, operator: P) -> Self::Operator;
}

/// Layer defined by a closure.
#[derive(Debug, Clone, Copy)]
pub struct LayerFn<F>(F);

impl<F, In, P, P2> Layer<In, P> for LayerFn<F>
where
    F: Fn(P) -> P2,
    P: ContextOperator<In>,
    P2: ContextOperator<In>,
{
    type Operator = P2;
    type Out = P2::Out;

    #[inline]
    fn layer(&self, operator: P) -> Self::Operator {
        (self.0)(operator)
    }
}

/// Create a layer from a closure.
pub fn layer_fn<F>(f: F) -> LayerFn<F> {
    LayerFn(f)
}

/// Extension trait for [`Layer`].
pub trait LayerExt<In, P>: Layer<In, P>
where
    P: ContextOperator<In>,
{
    /// Stack a outer layer on top of the inner layer.
    #[inline]
    fn with<Outer>(self, outer: Outer) -> Stack<Self, Outer>
    where
        Outer: Layer<In, Self::Operator>,
        Self: Sized,
    {
        stack::Stack(self, outer)
    }
}

impl<In, P, L> LayerExt<In, P> for L
where
    P: ContextOperator<In>,
    L: Layer<In, P>,
{
}
