use super::subscriber::Subscriber;

#[cfg(feature = "stream-publisher")]
pub use stream::stream;

/// Publisher implementation for streams.
#[cfg(feature = "stream-publisher")]
pub mod stream;

/// Publisher.
pub trait Publisher<'a> {
    /// Output.
    type Output;

    /// Subscribe.
    fn subscribe<S>(&mut self, subscriber: S)
    where
        S: Subscriber<Self::Output> + 'a;
}
