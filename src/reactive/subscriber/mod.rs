use super::subscription::Subscription;

/// Subscriber.
pub trait Subscriber<I> {
    /// Callback on subscribed.
    fn subscribed<S>(&mut self, subscription: S)
    where
        S: Subscription<Output = I>;
}
