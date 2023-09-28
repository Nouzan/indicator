use super::ContextOperator;

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
