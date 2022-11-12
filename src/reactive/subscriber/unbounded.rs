use core::future::ready;

use crate::reactive::{subscription::BoxSubscription, StreamError};

use super::{Complete, Subscriber};

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

    fn on_error(&mut self, error: StreamError) -> Complete<'_> {
        (self.0)(Err(error));
        Box::pin(ready(()))
    }

    fn on_complete(&mut self) -> Complete<'_> {
        Box::pin(ready(()))
    }
}

/// Create a unbounded subscriber.
pub fn unbounded<I, F>(f: F) -> Unbounded<F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    Unbounded(f, None)
}
