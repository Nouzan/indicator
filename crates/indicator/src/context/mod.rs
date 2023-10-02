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
        cache::CacheOperator,
        data::AddDataOperator,
        insert::{InsertDataOperator, InsertOperator, InsertWithDataOperator},
        inspect::InspectOperator,
    },
};

pub use self::{
    anymap::Context,
    layer::{
        cache::Cache,
        data::AddData,
        insert::{Insert, InsertData, InsertWithData},
        inspect::Inspect,
        layer_fn, Layer,
    },
    output::{output, output_with},
    value::{input, Value, ValueRef},
};

/// Operator that takes a [`Value`] as input and returns a [`Value`] as output.
/// And can be converted to an operator without the [`Value`] wrapper.
pub trait ContextOperator<In> {
    /// The output type inside the [`Value`] wrapper.
    type Out;

    /// Apply the operator.
    fn next(&mut self, input: Value<In>) -> Value<Self::Out>;
}

impl<In, P, Out> ContextOperator<In> for P
where
    P: Operator<Value<In>, Output = Value<Out>>,
{
    type Out = Out;

    #[inline]
    fn next(&mut self, input: Value<In>) -> Value<Out> {
        self.next(input)
    }
}

/// Extension trait for [`ContextOperator`].
pub trait ContextOperatorExt<In>: ContextOperator<In> {
    /// Add a layer.
    fn with<L>(self, layer: L) -> L::Operator
    where
        L: Layer<In, Self>,
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

    /// Add a [`Insert`] layer with the given [`RefOperator`] constructor
    /// (i.e. a function that returns a [`RefOperator`]).
    fn insert<R, Out>(self, f: impl Fn() -> R) -> InsertOperator<Self, R>
    where
        R: for<'a> RefOperator<'a, In, Output = Out>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(Insert(f))
    }

    /// Add a [`InsertData`] layer with the given [`RefOperator`] constructor.
    /// (i.e. a function that returns a [`RefOperator`]).
    fn insert_data<R, Out>(self, f: impl Fn() -> R) -> InsertDataOperator<Self, R>
    where
        R: for<'a> RefOperator<'a, In, Output = Option<Out>>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(InsertData(f))
    }

    /// Add a [`InsertWithData`] layer with the given [`RefOperator`] constructor.
    /// (i.e. a function that returns a [`RefOperator`]).
    fn insert_with_data<R, Env, Data>(self, f: impl Fn() -> R) -> InsertWithDataOperator<Self, R>
    where
        R: for<'a> RefOperator<'a, In, Output = (Env, Option<Data>)>,
        Env: Send + Sync + 'static,
        Data: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(InsertWithData(f))
    }

    /// Add an inspect layer with the given closure.
    fn inspect<F>(self, f: F) -> InspectOperator<Self, F>
    where
        F: Fn(ValueRef<'_, In>) + Clone,
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

impl<In, P> Operator<In> for ContextedOperator<P>
where
    P: ContextOperator<In>,
{
    type Output = P::Out;

    #[inline]
    fn next(&mut self, input: In) -> Self::Output {
        let mut value = self
            .0
            .next(Value::with_data(input, core::mem::take(&mut self.1)));
        core::mem::swap(&mut self.1, value.context_mut().data_mut());
        value.into_inner()
    }
}

/// Operator that takes a [`ValueRef`] as input.
pub trait RefOperator<'a, In> {
    /// The output type.
    type Output;

    /// Apply the operator.
    fn next(&mut self, input: ValueRef<'a, In>) -> Self::Output;
}

impl<'a, In, P> RefOperator<'a, In> for P
where
    P: Operator<ValueRef<'a, In>>,
    In: 'a,
{
    type Output = P::Output;

    #[inline]
    fn next(&mut self, input: ValueRef<'a, In>) -> Self::Output {
        self.next(input)
    }
}
