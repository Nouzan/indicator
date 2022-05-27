use crate::AsyncOperator;
use futures::FutureExt;

/// Convert Error.
#[derive(Debug, Clone, Copy)]
pub struct Map<F, P> {
    pub(super) inner: P,
    pub(super) f: F,
}

impl<I, F, P, O, E> AsyncOperator<I> for Map<F, P>
where
    P: AsyncOperator<I, Error = E>,
    F: FnMut(Result<P::Output, P::Error>) -> Result<O, E>,
{
    type Output = O;
    type Error = E;
    type Future<'a> = futures::future::Map<P::Future<'a>, &'a mut F> where P: 'a, F: 'a;

    fn poll_ready(
        &mut self,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn next(&mut self, input: I) -> Self::Future<'_> {
        self.inner.next(input).map(&mut self.f)
    }
}
