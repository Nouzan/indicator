use crate::reactive::{StreamError, Subscription};

use super::Subscriber;

/// A unbounded subscriber.
pub struct Unbounded<'a, F>(F, Option<Box<dyn Subscription + Send + 'a>>);

impl<'a, I, F> Subscriber<'a, I> for Unbounded<'a, F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    fn on_subscribe<S>(&mut self, mut subscription: S)
    where
        S: Subscription + 'a,
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
pub fn unbounded<'a, I, F>(f: F) -> Unbounded<'a, F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    Unbounded(f, None)
}
