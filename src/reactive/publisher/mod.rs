// use self::processed::Processed;

use super::{processor::Processor, subscriber::Subscriber};

// #[cfg(feature = "stream-publisher")]
// pub use stream::stream;

/// Publisher implementation for streams.
#[cfg(feature = "stream-publisher")]
pub mod stream;

// /// Processed Publisher.
// pub mod processed;

/// Publisher.
pub trait Publisher<'a> {
    /// Output.
    type Output;

    /// Subscribe.
    fn subscribe<S>(&mut self, subscriber: S)
    where
        S: Subscriber<Self::Output> + 'a;
}

impl<'a, 'b, P> Publisher<'a> for &'b mut P
where
    P: Publisher<'a>,
{
    type Output = P::Output;

    fn subscribe<S>(&mut self, subscriber: S)
    where
        S: Subscriber<Self::Output> + 'a,
    {
        Publisher::subscribe(&mut (**self), subscriber)
    }
}

// /// Extension of [`Publisher`].
// pub trait PublisherExt<'a>: Publisher<'a> {
//     /// Combine with a processor.
//     fn with<P>(&mut self, processor: P) -> Processed<'_, Self, P>
//     where
//         P: Processor<'a, Self::Output> + 'a,
//     {
//         Processed::new(self, processor)
//     }
// }

// impl<'a, P> PublisherExt<'a> for P where P: Publisher<'a> {}
