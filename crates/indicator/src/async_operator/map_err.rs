use crate::AsyncOperator;
use futures::TryFutureExt;

/// Convert Error.
#[derive(Debug, Clone, Copy)]
pub struct MapErr<F, P> {
    pub(super) inner: P,
    pub(super) f: F,
}

impl<I, E, F, P> AsyncOperator<I> for MapErr<F, P>
where
    P: AsyncOperator<I>,
    F: FnMut(P::Error) -> E,
{
    type Output = P::Output;
    type Error = E;
    type Future<'a> = futures::future::MapErr<P::Future<'a>, &'a mut F> where P: 'a, F: 'a;

    fn poll_ready(
        &mut self,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(&mut self.f)
    }

    fn next(&mut self, input: I) -> Self::Future<'_> {
        self.inner.next(input).map_err(&mut self.f)
    }
}
