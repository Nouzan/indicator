use core::num::NonZeroUsize;

/// Subscription.
/// Dropping a [`Subscrption`] should cause the subscription to cancel.
pub trait Subscription: Send + 'static {
    /// Request for some inputs.
    fn request(&mut self, num: NonZeroUsize);
    /// Request unbounded number of inputs.
    fn unbounded(&mut self);
}
