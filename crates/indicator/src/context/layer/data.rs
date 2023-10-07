use alloc::boxed::Box;

use crate::{
    context::{ContextOperator, Value},
    Operator,
};

use super::Layer;

/// Layer that adds data to the context.
pub struct AddData<T>(Box<dyn Fn() -> Option<T> + Send + Sync>);

impl<T> AddData<T>
where
    T: Send + Sync + 'static,
{
    /// Create [`AddData`] layer with the given data provider.
    pub fn new(provider: impl Fn() -> Option<T> + Send + Sync + 'static) -> Self {
        Self(Box::new(provider))
    }

    /// Create [`AddData`] layer with the given data.
    pub fn with_data(data: T) -> Self
    where
        T: Clone,
    {
        Self::new(move || Some(data.clone()))
    }

    /// Create [`AddData`] layer without data.
    /// That means to get the data from the context.
    pub fn from_context() -> Self {
        Self::new(|| None)
    }
}

impl<T, I, P> Layer<I, P> for AddData<T>
where
    P: ContextOperator<I>,
    T: Send + Sync + 'static,
{
    type Operator = AddDataOperator<T, P>;
    type Out = P::Out;

    fn layer(&self, operator: P) -> Self::Operator {
        AddDataOperator {
            data: (self.0)(),
            inner: operator,
        }
    }
}

/// Operator for [`AddData`].
/// It will add the given data to the data context at most once.
/// # Panic
/// Panic if the data is not in the context if it is not provided.
pub struct AddDataOperator<T, P> {
    data: Option<T>,
    inner: P,
}

impl<T, I, P> Operator<Value<I>> for AddDataOperator<T, P>
where
    P: ContextOperator<I>,
    T: Send + Sync + 'static,
{
    type Output = Value<P::Out>;

    #[inline]
    fn next(&mut self, mut input: Value<I>) -> Self::Output {
        if let Some(data) = self.data.take() {
            input.context_mut().data_mut().insert(data);
        } else if input.context().data().get::<T>().is_none() {
            panic!(
                "The `{}` is not in the context.",
                core::any::type_name::<T>()
            );
        }
        self.inner.next(input)
    }
}
