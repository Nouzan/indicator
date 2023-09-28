use super::ContextOperator;

/// Layer that caches the final context,
/// and provides it to the next evaluation.
pub mod cache;

/// Layer.
/// Convert an `In`-operator to another `In`-operator.
pub trait Layer<T, P>
where
    P: ContextOperator<T>,
{
    /// The output operator.
    type Output: ContextOperator<T>;

    /// Convert an `In`-operator to another `In`-operator.
    fn layer(&self, operator: P) -> Self::Output;
}

/// Layer defined by a closure.
#[derive(Debug, Clone, Copy)]
pub struct LayerFn<F>(F);

impl<F, T, P, P2> Layer<T, P> for LayerFn<F>
where
    F: Fn(P) -> P2,
    P: ContextOperator<T>,
    P2: ContextOperator<T>,
{
    type Output = P2;

    #[inline]
    fn layer(&self, operator: P) -> Self::Output {
        (self.0)(operator)
    }
}

/// Create a layer from a closure.
pub fn layer_fn<F>(f: F) -> LayerFn<F> {
    LayerFn(f)
}
