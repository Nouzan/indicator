use crate::AsyncOperator;
use core::task::{Context, Poll};
use futures::{
    future::{join, join_all, BoxFuture, JoinAll},
    ready,
};
use futures::{Future, FutureExt};

/// Convert Error.
#[derive(Debug, Clone, Copy)]
pub struct FacetVec<P, I> {
    pub(super) inner: P,
    phantom: core::marker::PhantomData<fn() -> I>,
}

impl<I, P, E, O> AsyncOperator<I> for FacetVec<P, I>
where
    P: IntoIterator + Send,
    <P as IntoIterator>::Item: AsyncOperator<I, Error = E, Output = O> + Send,
    for<'a> <<P as IntoIterator>::Item as AsyncOperator<I>>::Future<'a>: Send,
    I: Clone + Send,
    O: Send,
    E: Send,
{
    type Output = Vec<Result<O, E>>;
    type Error = E;
    type Future<'a> = BoxFuture<'a,Result<Self::Output,Self::Error>> where Self:'a;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for op in self.inner {
            match ready!(op.poll_ready(cx)) {
                Ok(_) => {}
                Err(err) => return Poll::Ready(Err(err)),
            }
        }
        Poll::Ready(Ok(()))
    }

    fn next(&mut self, input: I) -> Self::Future<'_> {
        join_all(
            self.inner
                .into_iter()
                .collect::<Vec<_>>()
                .iter_mut()
                .map(|op| op.next(input.clone()))
                .collect::<Vec<_>>(),
        )
        .map(|result| Ok(result))
        .boxed()
    }
}
