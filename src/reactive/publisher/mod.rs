use super::subscriber::Subscriber;
use core::future::Future;

#[cfg(feature = "stream-publisher")]
pub use stream::stream;

/// Publisher implementation for streams.
#[cfg(feature = "stream-publisher")]
pub mod stream;

/// Publisher.
pub trait Publisher {
    /// Output.
    type Output;

    /// Task.
    type Task<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    /// Subscribe.
    fn subscribe<'a, S>(self, subscriber: S) -> Self::Task<'a>
    where
        S: Subscriber<Self::Output> + 'a;
}
