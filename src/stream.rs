use crate::Operator;
use futures::task::{Context, Poll};
use futures::{ready, Stream};
use pin_project_lite::pin_project;

pin_project! {
    /// Operated.
    #[derive(Debug, Clone, Copy)]
    pub struct Operated<St, P> {
        #[pin]
        source: St,
        op: P,
    }
}

impl<St, P> Stream for Operated<St, P>
where
    St: Stream,
    P: Operator<St::Item>,
{
    type Item = P::Output;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let res = ready!(this.source.as_mut().poll_next(cx));
        Poll::Ready(res.map(|x| this.op.next(x)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.source.size_hint()
    }
}

/// Stream extension trait for indicators.
pub trait IndicatorStreamExt: Stream {
    /// Apply an indicator to the stream.
    fn indicator<P>(self, op: P) -> Operated<Self, P>
    where
        Self: Sized,
        P: Operator<Self::Item>,
    {
        Operated { source: self, op }
    }
}

impl<St: Stream> IndicatorStreamExt for St {}
