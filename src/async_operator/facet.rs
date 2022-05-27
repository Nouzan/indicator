use crate::{operator::facet::Facet, AsyncOperator};
use core::task::{Context, Poll};
use futures::{
    future::{join, BoxFuture},
    ready, FutureExt,
};

impl<'a, I, P1, P2, E> AsyncOperator<I> for Facet<I, P1, P2>
where
    P1: AsyncOperator<I, Error = E> + 'a,
    P1::Output: Send,
    for<'b> P1::Future<'b>: Send,
    P2: AsyncOperator<I, Error = E> + 'a,
    P2::Output: Send,
    for<'b> P2::Future<'b>: Send,
    E: Send,
    I: 'a + Clone,
{
    type Output = (P1::Output, P2::Output);
    type Error = E;
    type Future<'b> = BoxFuture<'b,Result<Self::Output,Self::Error>> where Self:'b;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match ready!(self.0.poll_ready(cx)) {
            Ok(_) => self.1.poll_ready(cx),
            Err(err) => Poll::Ready(Err(err)),
        }
    }

    fn next(&mut self, input: I) -> Self::Future<'_> {
        let fut1 = self.0.next(input.clone());
        let fut2 = self.1.next(input);
        join(fut1, fut2)
            .map(|(x, y)| match (x, y) {
                (Ok(a), Ok(b)) => Ok((a, b)),
                (Err(err), _) => Err(err),
                (_, Err(err)) => Err(err),
            })
            .boxed()
    }
}
