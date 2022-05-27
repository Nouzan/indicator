use crate::{
    operator::{facet::Facet, facet::FacetMap},
    AsyncOperator,
};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures::{
    future::{join, BoxFuture},
    ready, FutureExt,
};
use pin_project_lite::pin_project;
use std::{collections::HashMap, hash::Hash};

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

/// Combine two operators into a [`Facet`] operator.
pub fn facet<I, P1, P2>(op1: P1, op2: P2) -> Facet<I, P1, P2> {
    Facet(op1, op2, core::marker::PhantomData::default())
}

impl<'a, I, K, V, E> AsyncOperator<I> for FacetMap<I, K, V>
where
    K: Eq + Hash + Clone,
    V: AsyncOperator<I, Error = E> + 'a,
    I: 'a + Clone,
{
    type Output = HashMap<K, V::Output>;
    type Error = E;
    type Future<'b> = BoxFuture<'b,Result<Self::Output,Self::Error>> where Self:'b;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for (key, op) in self.0 {
            match ready!(op.poll_ready(cx)) {
                Ok(_) => {}
                Err(err) => return Poll::Ready(Err(err)),
            }
        }
        Poll::Ready(Ok(()))
    }

    fn next(&mut self, input: I) -> Self::Future<'_> {
        self.0
            .iter_mut()
            .map(|(k, v)| (k.clone(), v.next(input.clone())))
            .collect()
    }
}
