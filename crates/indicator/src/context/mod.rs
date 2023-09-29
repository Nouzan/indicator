mod anymap;

/// Value with context.
pub mod value;

/// Extractors.
pub mod extractor;

/// Convert an `In`-operator to another `In`-operator.
pub mod layer;

/// Output operator.
pub mod output;

use crate::Operator;

use self::layer::{cache::CacheOperator, insert::InsertOperator};

pub use self::{
    anymap::Context,
    layer::{cache::Cache, insert::Insert, layer_fn, Layer},
    output::{output, output_with},
    value::{input, IntoValue, Value, ValueRef},
};

/// Operator that takes a `Value` as input and returns a `Value` as output.
/// And can be converted to an operator without the `Value` wrapper.
pub trait ContextOperator<T> {
    /// The output type.
    /// Just an alias for `Self::Output`.
    type Output: IntoValue;

    /// Apply the operator.
    fn next(&mut self, input: Value<T>) -> Self::Output;
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

/// Extension trait for [`ContextOperator`].
pub trait ContextOperatorExt<T>: ContextOperator<T> {
    /// Add a layer.
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

    /// Add a cache layer with the given `length`.
    /// # Panic
    /// Panic if the length is 0.
    fn cache(self, length: usize) -> CacheOperator<Self>
    where
        Self: Sized,
    {
        self.with(Cache::with_length(
            length.try_into().expect("`length` cannot be 0"),
        ))
    }

    /// Add a insert layer with the given [`RefOperator`] constructor
    /// (i.e. a function that returns a [`RefOperator`]).
    fn insert<R, Out>(self, f: impl Fn() -> R) -> InsertOperator<Self, R>
    where
        R: for<'a> RefOperator<'a, T, Output = Out>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(Insert(f))
    }
}

impl<T, P> ContextOperatorExt<T> for P where P: ContextOperator<T> {}

/// Contexted Operator.
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

/// Operator that takes a [`ValueRef`] as input.
pub trait RefOperator<'a, T> {
    /// The output type.
    type Output;

    /// Apply the operator.
    fn next(&mut self, input: ValueRef<'a, T>) -> Self::Output;
}

impl<'a, T, P> RefOperator<'a, T> for P
where
    P: Operator<ValueRef<'a, T>>,
    T: 'a,
{
    type Output = P::Output;

    #[inline]
    fn next(&mut self, input: ValueRef<'a, T>) -> Self::Output {
        self.next(input)
    }
}
