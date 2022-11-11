use crate::reactive::{subscription::BoxSubscription, StreamError};

use super::Subscriber;

/// A unbounded subscriber.
pub struct Unbounded<F>(F, Option<BoxSubscription>);

impl<I, F> Subscriber<I> for Unbounded<F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    fn on_subscribe(&mut self, mut subscription: BoxSubscription) {
        subscription.unbounded();
        self.1 = Some(subscription);
    }

    fn on_next(&mut self, input: I) {
        (self.0)(Ok(input));
    }

    fn on_error(&mut self, error: StreamError) {
        (self.0)(Err(error))
    }

    fn on_complete(&mut self) {}
}

/// Create a unbounded subscriber.
pub fn unbounded<I, F>(f: F) -> Unbounded<F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    Unbounded(f, None)
}
