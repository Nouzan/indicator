use super::AsyncOperator;
use crate::Operator;
use core::convert::Infallible;
use core::task::{Context, Poll};
use futures::future::{ready, Ready};

/// Next operator that converts a blocking [`Operator`] into an [`AsyncOperator`].
#[derive(Debug, Clone, Copy)]
pub struct Next<P> {
    pub(crate) inner: P,
}

impl<P, I, O> AsyncOperator<I> for Next<P>
where
    P: Operator<I, Output = O>,
{
    type Output = O;

    type Error = Infallible;

    type Future<'a> = Ready<Result<Self::Output, Self::Error>> where P: 'a;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn next(&mut self, input: I) -> Self::Future<'_> {
        ready(Ok(Operator::next(&mut self.inner, input)))
    }
}

/// Convert a "sync" opeartor into a async operator.
pub fn next<I, P>(operator: P) -> Next<P>
where
    P: Operator<I>,
{
    Next { inner: operator }
}
