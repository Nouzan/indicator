use crate::reactive::{processor::Processor, StreamError, Subscriber};

use super::Publisher;

/// [`Publisher`] for [`with`](super::Publisher::with) method.
#[derive(Debug)]
pub struct Processed<'a, T: ?Sized, U> {
    publisher: &'a mut T,
    processor: Option<U>,
}

impl<'a, 'b, T, U> Processed<'b, T, U>
where
    T: Publisher<'a> + ?Sized,
    U: Processor<'a, T::Output> + 'a,
{
    pub(super) fn new(publisher: &'b mut T, processor: U) -> Self {
        Self {
            publisher,
            processor: Some(processor),
        }
    }
}

impl<'a, 'b, T, U> Publisher<'a> for Processed<'b, T, U>
where
    T: Publisher<'a>,
    U: Processor<'a, T::Output> + 'a,
{
    type Output = U::Output;

    fn subscribe<S>(&mut self, mut subscriber: S)
    where
        S: Subscriber<Self::Output> + 'a,
    {
        if let Some(mut processor) = self.processor.take() {
            processor.subscribe(subscriber);
            self.publisher.subscribe(processor);
        } else {
            subscriber.on_error(StreamError::abort("`Processed` can only be subsribed once"));
        }
    }
}
