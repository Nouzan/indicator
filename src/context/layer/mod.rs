use super::InOperator;

/// Layer.
/// Convert an `In`-operator to another `In`-operator.
pub trait Layer<T, P>
where
    P: InOperator<T>,
{
    /// The output operator.
    type Output: InOperator<T>;

    /// Convert an `In`-operator to another `In`-operator.
    fn layer(&self, operator: P) -> Self::Output;
}
