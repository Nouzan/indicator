use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::{
    reactive::{Publisher, StreamError, Subscriber},
    Operator,
};

pin_project! {
    /// Operator Processor.
    pub struct OperatorProcessor<'a, I, P, O> {
        op: P,
        subscriber: Option<Pin<Box<dyn Subscriber<O> + 'a>>>,
        _input: PhantomData<fn() -> I>,
    }
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
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<bool, StreamError>> {
        self.project()
            .subscriber
            .as_mut()
            .ok_or_else(|| StreamError::unknown("`OperatorProcessor` must subscribe first"))?
            .as_mut()
            .poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), StreamError> {
        let this = self.project();
        let next = this.op.next(item);
        this.subscriber
            .as_mut()
            .ok_or_else(|| StreamError::unknown("`OperatorProcessor` must subscribe first"))?
            .as_mut()
            .start_send(next)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<bool, StreamError>> {
        self.project()
            .subscriber
            .as_mut()
            .ok_or_else(|| StreamError::unknown("`OperatorProcessor` must subscribe first"))?
            .as_mut()
            .poll_flush(cx)
    }

    fn closing(self: Pin<&mut Self>, reason: Result<(), StreamError>) -> Result<(), StreamError> {
        self.project()
            .subscriber
            .as_mut()
            .ok_or_else(|| StreamError::unknown("`OperatorProcessor` must subscribe first"))?
            .as_mut()
            .closing(reason)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StreamError>> {
        self.project()
            .subscriber
            .as_mut()
            .ok_or_else(|| StreamError::unknown("`OperatorProcessor` must subscribe first"))?
            .as_mut()
            .poll_close(cx)
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
        self.subscriber = Some(Box::pin(subscriber));
    }
}
