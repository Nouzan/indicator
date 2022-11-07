use core::num::NonZeroUsize;

/// Subscription.
pub trait Subscription {
    /// Request for some inputs.
    fn request(&mut self, num: NonZeroUsize);
    /// Cancel the stream.
    fn cancel(&mut self);
}
