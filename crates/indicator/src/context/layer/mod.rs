use self::stack::Stack;

use super::{
    AddData, ContextOperator, Insert, InsertData, InsertWithData, Inspect, RefOperator, ValueRef,
};

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
        Stack(self, outer)
    }

    /// Add a [`Insert`] layer with the given [`RefOperator`] constructor
    /// (i.e. a function that returns a [`RefOperator`]).
    fn insert<R, Out, F>(self, f: F) -> Stack<Self, Insert<F>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = Out>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(Insert(f))
    }

    /// Add a [`InsertData`] layer with the given [`RefOperator`] constructor.
    /// (i.e. a function that returns a [`RefOperator`]).
    fn insert_data<R, Out, F>(self, f: F) -> Stack<Self, InsertData<F>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = Option<Out>>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(InsertData(f))
    }

    /// Add a [`InsertWithData`] layer with the given [`RefOperator`] constructor.
    /// (i.e. a function that returns a [`RefOperator`]).
    fn insert_with_data<R, Env, Data, F>(self, f: F) -> Stack<Self, InsertWithData<F>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = (Env, Option<Data>)>,
        Env: Send + Sync + 'static,
        Data: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(InsertWithData(f))
    }

    /// Add an inspect layer with the given closure.
    fn inspect<F>(self, f: F) -> Stack<Self, Inspect<F>>
    where
        F: Fn(ValueRef<'_, In>) + Clone,
        Self: Sized,
    {
        self.with(Inspect(f))
    }

    /// Provide data to the context.
    fn provide<D>(self, data: D) -> Stack<Self, AddData<D>>
    where
        D: Clone + Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::with_data(data))
    }

    /// Provide data to the context with the given data provider.
    fn provide_with<D>(self, provider: impl Fn() -> Option<D> + 'static) -> Stack<Self, AddData<D>>
    where
        D: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::new(provider))
    }

    /// Declare that the data of type `D` is in the context.
    fn from_context<D>(self) -> Stack<Self, AddData<D>>
    where
        D: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::<D>::from_context())
    }
}

impl<In, P, L> LayerExt<In, P> for L
where
    P: ContextOperator<In>,
    L: Layer<In, P>,
{
}
