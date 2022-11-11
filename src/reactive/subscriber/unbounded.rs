use crate::reactive::{StreamError, Subscription};

use super::Subscriber;

/// A unbounded subscriber.
pub struct Unbounded<F>(F, Option<Box<dyn Subscription + Send>>);

impl<I, F> Subscriber<I> for Unbounded<F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    fn on_subscribe<S>(&mut self, mut subscription: S)
    where
        S: Subscription,
    {
        subscription.unbounded();
        self.1 = Some(Box::new(subscription));
    }

    fn on_next(&mut self, input: I) {
        (self.0)(Ok(input));
    }

    fn on_error<E>(mut self, error: E)
    where
        E: Into<StreamError>,
    {
        (self.0)(Err(error.into()))
    }

    fn on_complete(self) {}
}

/// Create a unbounded subscriber.
pub fn unbounded<I, F>(f: F) -> Unbounded<F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    Unbounded(f, None)
}
