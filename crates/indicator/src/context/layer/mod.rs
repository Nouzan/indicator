use alloc::sync::Arc;

use self::{optional::OptionalLayer, stack::Stack, then::Then};

use super::{
    AddData, BoxContextOperator, Context, ContextOperator, ContextOperatorExt, Insert, InsertData,
    InsertWithData, Inspect, RefOperator,
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

/// Then layer.
pub mod then;

/// Optional layer.
pub mod optional;

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
    /// Create a boxed [`Layer`].
    fn boxed(self) -> BoxLayer<P, In, Self::Out>
    where
        Self: Sized + Send + Sync + 'static,
        Self::Operator: Send + 'static,
    {
        BoxLayer::new(self)
    }

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
    ///
    /// We use this method to add the output of the operator to the `env` context.
    fn insert_env<R, Out, F>(self, f: F) -> Stack<Self, Insert<F>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = Out>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(Insert(f))
    }

    /// Optionally add the [`Insert`] layer with the given [`RefOperator`] constructor.
    ///
    /// See [`insert_env`] for more details.
    fn insert_env_if<R, Out, F>(self, enable: bool, f: F) -> Stack<Self, OptionalLayer<Insert<F>>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = Out>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with_if(if enable { Some(Insert(f)) } else { None })
    }

    /// Add a [`InsertData`] layer with the given [`RefOperator`] constructor.
    /// (i.e. a function that returns a [`RefOperator`]).
    ///
    /// We use this method to add the output of the operator to the `data` context.
    fn insert_data<R, Out, F>(self, f: F) -> Stack<Self, InsertData<F>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = Option<Out>>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(InsertData(f))
    }

    /// Optionally add the [`InsertData`] layer with the given [`RefOperator`] constructor.
    ///
    /// See [`insert_data`] for more details.
    fn insert_data_if<R, Out, F>(
        self,
        enable: bool,
        f: F,
    ) -> Stack<Self, OptionalLayer<InsertData<F>>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = Option<Out>>,
        Out: Send + Sync + 'static,
        Self: Sized,
    {
        self.with_if(if enable { Some(InsertData(f)) } else { None })
    }

    /// Add a [`InsertWithData`] layer with the given [`RefOperator`] constructor.
    /// (i.e. a function that returns a [`RefOperator`]).
    ///
    /// We use this method to add the output of the operator to the `env` and `data` context simultaneously.
    fn insert<R, Env, Data, F>(self, f: F) -> Stack<Self, InsertWithData<F>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = (Env, Option<Data>)>,
        Env: Send + Sync + 'static,
        Data: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(InsertWithData(f))
    }

    /// Optionally add the [`InsertWithData`] layer with the given [`RefOperator`] constructor.
    ///
    /// See [`insert`] for more details.
    fn insert_if<R, Env, Data, F>(
        self,
        enable: bool,
        f: F,
    ) -> Stack<Self, OptionalLayer<InsertWithData<F>>>
    where
        F: Fn() -> R,
        R: for<'a> RefOperator<'a, In, Output = (Env, Option<Data>)>,
        Env: Send + Sync + 'static,
        Data: Send + Sync + 'static,
        Self: Sized,
    {
        self.with_if(if enable {
            Some(InsertWithData(f))
        } else {
            None
        })
    }

    /// Add an inspect layer with the given closure.
    fn inspect<F>(self, f: F) -> Stack<Self, Inspect<F>>
    where
        F: Fn(&In, &Context) + Clone,
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

    /// Optionally provide data to the context.
    ///
    /// If the data is `None`, the layer will be skipped,
    /// so will not panic if the data is not in the context.
    fn provide_if<D>(self, data: Option<D>) -> Stack<Self, OptionalLayer<AddData<D>>>
    where
        D: Clone + Send + Sync + 'static,
        Self: Sized,
    {
        self.with_if(data.map(AddData::with_data))
    }

    /// Provide data to the context with the given data provider.
    fn provide_with<D>(
        self,
        provider: impl Fn() -> Option<D> + Send + Sync + 'static,
    ) -> Stack<Self, AddData<D>>
    where
        D: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::new(provider))
    }

    /// Optionally provide data to the context with the given data provider.
    ///
    /// If not enabled, the layer will be skipped,
    /// so will not panic if the data is not in the context.
    fn provide_with_if<D>(
        self,
        enable: bool,
        provider: impl Fn() -> Option<D> + Send + Sync + 'static,
    ) -> Stack<Self, OptionalLayer<AddData<D>>>
    where
        D: Send + Sync + 'static,
        Self: Sized,
    {
        self.with_if(if enable {
            Some(AddData::new(provider))
        } else {
            None
        })
    }

    /// Declare that the data of type `D` is in the context.
    // FIXME: this method should be removed in the future.
    #[allow(clippy::wrong_self_convention)]
    #[deprecated(note = "use `data_from_context` instead")]
    fn from_context<D>(self) -> Stack<Self, AddData<D>>
    where
        D: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::<D>::from_context())
    }

    /// Declare that the data of type `D` is in the data context.
    fn data_from_context<D>(self) -> Stack<Self, AddData<D>>
    where
        D: Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::<D>::from_context())
    }

    /// Add a closure that will be called after the operator is evaluated.
    fn then_with<Out, F, Builder>(self, builder: Builder) -> Stack<Self, Then<Builder>>
    where
        Builder: Fn() -> F,
        F: FnMut(Self::Out, &Context) -> Out + Clone,
        Self: Sized,
    {
        self.with(Then(builder))
    }

    /// Optionally add a layer.
    fn with_if<L>(self, layer: Option<L>) -> Stack<Self, OptionalLayer<L>>
    where
        L: Layer<In, Self::Operator>,
        L::Operator: ContextOperator<In, Out = Self::Out>,
        Self: Sized,
    {
        self.with(OptionalLayer(layer))
    }
}

impl<In, P, L> LayerExt<In, P> for L
where
    P: ContextOperator<In>,
    L: Layer<In, P>,
{
}

/// A boxed [`Layer`].
pub struct BoxLayer<P, In, Out> {
    inner: Arc<
        dyn Layer<In, P, Operator = BoxContextOperator<In, Out>, Out = Out> + Send + Sync + 'static,
    >,
}

impl<P, In, Out> BoxLayer<P, In, Out> {
    /// Create a boxed [`Layer`].
    pub fn new<L>(inner: L) -> Self
    where
        P: ContextOperator<In>,
        L: Layer<In, P, Out = Out> + Send + Sync + 'static,
        L::Operator: Send + 'static,
    {
        let layer = layer_fn(move |op: P| inner.layer(op).boxed());
        Self {
            inner: Arc::new(layer),
        }
    }
}

impl<P, In, Out> Layer<In, P> for BoxLayer<P, In, Out>
where
    P: ContextOperator<In>,
{
    type Operator = BoxContextOperator<In, Out>;

    type Out = Out;

    fn layer(&self, operator: P) -> Self::Operator {
        self.inner.layer(operator)
    }
}
