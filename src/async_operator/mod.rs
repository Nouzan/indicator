use crate::operator::then::Then;
use core::future::Future;
use core::marker::PhantomData;
use core::task::{Context, Poll};

#[cfg(feature = "tower")]
/// [tower_service::Service] as [`AsyncOperator`].
pub mod tower;

/// Next operator, the container of "sync" operators.
pub mod next;

/// And then.
pub mod and_then;

/// Map error.
pub mod map_err;

/// Map
pub mod map;

/// Facet
pub mod facet;

/// Facet Tuple
pub mod facet_vec;

pub use next::{next, Next};
#[cfg(feature = "tower")]
pub use tower::{ServiceOp, ServiceOperator};

use self::map_err::MapErr;

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
    /// And then.
    fn and_then<P2>(self, other: P2) -> Then<I, Self, P2>
    where
        Self: Sized,
        P2: AsyncOperator<Self::Output>,
    {
        Then(self, other, PhantomData)
    }

    /// Convert error.
    fn map_err<E, F>(self, f: F) -> MapErr<F, Self>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapErr { inner: self, f }
    }
}

impl<I, P> AsyncOperatorExt<I> for P where P: AsyncOperator<I> {}
