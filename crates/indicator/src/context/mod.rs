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

use self::{
    anymap::Map,
    layer::{
        cache::CacheOperator, data::AddDataOperator, insert::InsertOperator,
        inspect::InspectOperator,
    },
};

pub use self::{
    anymap::Context,
    layer::{cache::Cache, data::AddData, insert::Insert, inspect::Inspect, layer_fn, Layer},
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
    #[inline]
    fn finish(self) -> ContextedOperator<Self>
    where
        Self: Sized,
    {
        self.finish_with_data(Map::default())
    }

    /// Build into an operator without the `Value` wrapper with the given data context.
    #[inline]
    fn finish_with_data(self, data: Map) -> ContextedOperator<Self>
    where
        Self: Sized,
    {
        ContextedOperator(self, data)
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

    /// Add an inspect layer with the given closure.
    fn inspect<F>(self, f: F) -> InspectOperator<Self, F>
    where
        F: Fn(ValueRef<'_, T>) + Clone,
        Self: Sized,
    {
        self.with(Inspect(f))
    }

    /// Provide data to the context.
    fn provide<D>(self, data: D) -> AddDataOperator<D, Self>
    where
        D: Clone + Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::with_data(data))
    }

    /// Provide data to the context with the given data provider.
    fn provide_with<D>(self, provider: impl Fn() -> Option<D> + 'static) -> AddDataOperator<D, Self>
    where
        D: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::new(provider))
    }

    /// Declare that the data of type `D` is in the context.
    fn from_context<D>(self) -> AddDataOperator<D, Self>
    where
        D: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::<D>::from_context())
    }
}

impl<T, P> ContextOperatorExt<T> for P where P: ContextOperator<T> {}

/// Contexted Operator.
#[derive(Debug, Default)]
pub struct ContextedOperator<P>(P, Map);

impl<T, P> Operator<T> for ContextedOperator<P>
where
    P: ContextOperator<T>,
{
    type Output = <<P as ContextOperator<T>>::Output as IntoValue>::Inner;

    #[inline]
    fn next(&mut self, input: T) -> Self::Output {
        let mut value = self
            .0
            .next(Value::with_data(input, core::mem::take(&mut self.1)))
            .into_value();
        core::mem::swap(&mut self.1, value.context_mut().data_mut());
        value.into_inner()
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
