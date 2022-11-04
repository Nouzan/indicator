use super::subscriber::Subscriber;

/// Publisher.
pub trait Publisher {
    /// Output.
    type Output;

    /// Subscribe.
    fn subscribe<S>(self, subscriber: S)
    where
        S: Subscriber<Self::Output>;
}
