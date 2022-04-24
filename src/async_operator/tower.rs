use super::AsyncOperator;
use core::task::{Context, Poll};
use tower_service::Service;

/// A wrapper converting a [`Service`] into an [`AsyncOperator`].
#[derive(Debug, Clone, Copy)]
pub struct ServiceOp<S> {
    inner: S,
    // _input: PhantomData<fn() -> I>,
}

impl<S> ServiceOp<S> {
    /// Create from a service.
    fn new<I>(inner: S) -> Self
    where
        S: Service<I>,
    {
        Self { inner }
    }
}

impl<S, I, O> AsyncOperator<I> for ServiceOp<S>
where
    S: Service<I, Response = O>,
{
    type Output = O;

    type Error = S::Error;

    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn next(&mut self, input: I) -> Self::Future {
        self.inner.call(input)
    }
}

/// A trait that converts a [`Service`] into an [`AsyncOperator`]
pub trait ServiceOperator<I>: Service<I> {
    /// Convert into an [`AsyncOperator`].
    fn into_async_operator(self) -> ServiceOp<Self>
    where
        Self: Sized,
    {
        ServiceOp::new(self)
    }
}

impl<T, I> ServiceOperator<I> for T where T: Service<I> {}
