use super::{error::StreamError, subscription::BoxSubscription};

pub use self::unbounded::unbounded;

#[cfg(feature = "task-subscriber")]
pub use self::task::subscriber_fn;

/// Unbounded Subscriber.
pub mod unbounded;

/// Task subscriber.
#[cfg(feature = "task-subscriber")]
pub mod task;

/// Subscriber.
pub trait Subscriber<I>: Send {
    /// Callback on subscribed.
    fn on_subscribe(&mut self, subscription: BoxSubscription);
    /// Callback on receiving the next input.
    fn on_next(&mut self, input: I);
    /// Callback on error.
    fn on_error(&mut self, error: StreamError);
    /// Calllback on complete.
    fn on_complete(&mut self);
}
