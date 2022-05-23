use crate::{operator::then::Then, AsyncOperator};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::{pin_mut, ready};
use pin_project_lite::pin_project;

pin_project! {
    /// Future for [`Then`].
    pub struct AndThenFuture<'a, Fut, P>{
        #[pin]
        input_fut: Fut,
        output_op: &'a mut P,
    }
}

impl<'a, T, E, Fut, P> Future for AndThenFuture<'a, Fut, P>
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
            }
            Err(err) => Poll::Ready(Err(err)),
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
    type Future<'a> = AndThenFuture<'a, P1::Future<'a>, P2> where I: 'a, P1: 'a, P2: 'a;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match ready!(self.0.poll_ready(cx)) {
            Ok(_) => self.1.poll_ready(cx),
            Err(err) => Poll::Ready(Err(err)),
        }
    }

    fn next(&mut self, input: I) -> Self::Future<'_> {
        let input_fut = self.0.next(input);
        AndThenFuture {
            input_fut,
            output_op: &mut self.1,
        }
    }
}
