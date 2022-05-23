use crate::operator::then::Then;
use core::future::Future;
use core::marker::PhantomData;
use core::task::{Context, Poll};

#[cfg(feature = "tower")]
/// [tower_service::Service] as [`AsyncOperator`].
pub mod tower;

/// Then.
pub mod then;

/// Next operator, the container of "sync" operators.
pub mod next;

pub use next::{next, Next};
#[cfg(feature = "tower")]
pub use tower::{ServiceOp, ServiceOperator};

/// Async Operator.
/// It can be seen as an alias of [`tower_service::Service`].
pub trait AsyncOperator<I> {
    /// Output type.
    type Output;

    /// Error type.
    type Error;

    /// The future output value.
    type Future<'a>: Future<Output = Result<Self::Output, Self::Error>>
    where
        Self: 'a;

    /// Check if the operator is ready to process the next input.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Process the next input.
    fn next(&mut self, input: I) -> Self::Future<'_>;
}

/// Extention trait for async operators.
pub trait AsyncOperatorExt<I>: AsyncOperator<I> {
    /// Then.
    fn then<P2>(self, other: P2) -> Then<I, Self, P2>
    where
        Self: Sized,
        P2: AsyncOperator<Self::Output>,
    {
        Then(self, other, PhantomData)
    }
}

impl<I, P> AsyncOperatorExt<I> for P where P: AsyncOperator<I> {}
