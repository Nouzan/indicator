use core::{future::ready, marker::PhantomData, pin::Pin, task::{Context, Poll}};

use crate::{
    reactive::{subscription::BoxSubscription, Publisher, StreamError, Subscriber},
    Operator,
};

/// Operator Processor.
pub struct OperatorProcessor<'a, I, P, O> {
    op: P,
    subscriber: Option<Box<dyn Subscriber<O> + 'a>>,
    _input: PhantomData<fn() -> I>,
}

impl<'a, I, P, O> OperatorProcessor<'a, I, P, O>
where
    P: Operator<I, Output = O> + Send,
{
    /// Create a new [`OperatorProcessor`] from the given operator.
    pub fn new(op: P) -> Self {
        Self {
            op,
            subscriber: None,
            _input: PhantomData::default(),
        }
    }
}

impl<'a, I, P, O> Subscriber<I> for OperatorProcessor<'a, I, P, O>
where
    P: Operator<I, Output = O> + Send,
{
    fn on_subscribe(&mut self, subscription: BoxSubscription) {
        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_subscribe(subscription);
        }
    }

    fn on_next(&mut self, input: I) {
        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_next(self.op.next(input));
        }
    }

    fn on_error(&mut self, error: StreamError) -> Complete<'_> {
        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_error(error)
        } else {
            Box::pin(ready(()))
        }
    }

    fn on_complete(&mut self) -> Complete<'_> {
        if let Some(subscriber) = self.subscriber.as_mut() {
            subscriber.on_complete()
        } else {
            Box::pin(ready(()))
        }
    }

    fn poll_ready(self: Pin<&mut Self>, cx: Context<'_>) -> Poll<Result<(), StreamError>> {
        todo!()
    }

    fn feed_next(self: Pin<&mut Self>, item: I) -> Result<(), StreamError> {
        todo!()
    }

    fn poll_finish(
        self: Pin<&mut Self>,
        cx: Context<'_>,
        reason: Result<(), StreamError>,
    ) -> Poll<Result<(), StreamError>> {
        todo!()
    }
}

impl<'a, I, P, O> Publisher<'a> for OperatorProcessor<'a, I, P, O>
where
    P: Operator<I, Output = O>,
{
    type Output = O;

    fn subscribe<S>(&mut self, subscriber: S)
    where
        S: Subscriber<Self::Output> + 'a,
    {
        self.subscriber = Some(Box::new(subscriber));
    }
}
