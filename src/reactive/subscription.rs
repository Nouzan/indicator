use core::num::NonZeroUsize;

/// Boxed Subscription.
pub type BoxSubscription = Box<dyn Subscription>;

/// Subscription.
/// Dropping a [`Subscrption`] should cause the subscription to cancel.
pub trait Subscription: Send + 'static {
    /// Request for some inputs.
    fn request(&self, num: NonZeroUsize);
    /// Request unbounded number of inputs.
    fn unbounded(&self);
    /// Convert into a [`BoxSubscription`]
    fn boxed(self) -> BoxSubscription
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
