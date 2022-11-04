use core::num::NonZeroUsize;

use futures::{Sink, TryStream};

/// Subscription.
pub trait Subscription:
    Sink<NonZeroUsize, Error = Self::Err> + TryStream<Ok = Self::Output, Error = Self::Err>
{
    /// Output.
    type Output;

    /// Error.
    type Err;
}
