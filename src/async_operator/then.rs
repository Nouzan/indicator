use crate::{operator::then::Then, AsyncOperator};
use core::marker::PhantomData;
use core::future::Future;
use core::task::{Poll, Context};
use core::pin::Pin;
use futures::future::{BoxFuture, MapOk, AndThen};
use futures::{ready, TryFutureExt, pin_mut};
use pin_project_lite::pin_project;

pin_project! {
    /// Future for [`Then`].
    pub struct ThenFuture<'a, Fut, P>{
        #[pin]
        input_fut: Fut,
        output_op: &'a mut P,
    }
}

impl<'a, T, E, Fut, P> Future for ThenFuture<'a, Fut, P>
where
    Fut: Future<Output = Result<T, E>>,
    P: AsyncOperator<T, Error = E>,
{
    type Output = Result<P::Output, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match ready!(this.input_fut.poll(cx)) {
            Ok(input) => {
                let fut = this.output_op.next(input);
                pin_mut!(fut);
                fut.poll(cx)
            },
            Err(err) => {
                Poll::Ready(Err(err))
            }
        }
    }
}

impl<I, E, P1, P2> AsyncOperator<I> for Then<I, P1, P2>
where
    P1: AsyncOperator<I, Error = E>,
    P2: AsyncOperator<P1::Output, Error = E>,
{
    type Output = P2::Output;
    type Error = E;
    type Future = AndThen<P1::Future, P2::Future, fn(P1::Output) -> P2::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match ready!(self.0.poll_ready(cx)) {
            Ok(_) => self.1.poll_ready(cx),
            Err(err) => Poll::Ready(Err(err)),
        }
    }

    fn next(&mut self, input: I) -> Self::Future {
        self
            .0
            .next(input)
            .and_then(self.1.next)
    }
}
