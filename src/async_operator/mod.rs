use super::Operator;
use core::convert::Infallible;
use core::future::Future;
use core::task::{Context, Poll};
use futures::future::{ready, Ready};

/// Async Operator.
/// It can be seen as an alias of [`tower_service::Service`].
pub trait AsyncOperator<I> {
    /// Output type.
    type Output;

    /// Error type.
    type Error;

    /// The future output value.
    type Future: Future<Output = Result<Self::Output, Self::Error>>;

    /// Check if the operator is ready to process the next input.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Process the next input.
    fn next(&mut self, input: I) -> Self::Future;
}

/// Next operator that converts a blocking [`Operator`] into an [`AsyncOperator`].
#[derive(Debug, Clone, Copy)]
pub struct Next<P> {
    pub(crate) inner: P,
}

impl<P, I, O> AsyncOperator<I> for Next<P>
where
    P: Operator<I, Output = O>,
{
    type Output = O;

    type Error = Infallible;

    type Future = Ready<Result<Self::Output, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn next(&mut self, input: I) -> Self::Future {
        ready(Ok(Operator::next(&mut self.inner, input)))
    }
}

#[cfg(feature = "tower")]
pub use tower::{ServiceOp, ServiceOperator};

#[cfg(feature = "tower")]
/// [tower_service::Service] as [`AsyncOperator`].
pub mod tower;
