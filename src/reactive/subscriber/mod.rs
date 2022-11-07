use super::{error::StreamError, subscription::Subscription};

pub use self::unbounded::unbounded;

/// Unbounded Subscriber.
pub mod unbounded;

/// Subscriber.
pub trait Subscriber<'a, I>: Send {
    /// Callback on subscribed.
    fn on_subscribe<S>(&mut self, subscription: S)
    where
        S: Subscription + 'a;
    /// Callback on receiving the next input.
    fn on_next(&mut self, input: I);
    /// Callback on error.
    fn on_error<E>(self, error: E)
    where
        E: Into<StreamError>;
    /// Calllback on complete.
    fn on_complete(self);
}
