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

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let res = ready!(this.source.as_mut().poll_next(cx));
        Poll::Ready(res.map(|x| this.op.next(x)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.source.size_hint()
    }
}

#[cfg(feature = "async")]
/// Async version of [`Operated`].
pub mod async_operated {
    use crate::async_operator::AsyncOperator;
    use futures::task::{Context, Poll};
    use futures::{future::Future, pin_mut, ready, Stream};
    use pin_project_lite::pin_project;

    pin_project! {
        /// Operated.
        #[derive(Debug, Clone, Copy)]
        pub struct Operated<St, P> {
            #[pin]
            pub(crate) source: St,
            pub(crate) op: P,
        }
    }

    impl<St, P> Stream for Operated<St, P>
    where
        St: Stream,
        P: AsyncOperator<St::Item>,
    {
        type Item = Result<P::Output, P::Error>;

        fn poll_next(
            self: core::pin::Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Self::Item>> {
            let mut this = self.project();
            match ready!(this.source.as_mut().poll_next(cx)) {
                Some(input) => match ready!(this.op.poll_ready(cx)) {
                    Ok(()) => {
                        let fut = this.op.next(input);
                        pin_mut!(fut);
                        let next = ready!(fut.poll(cx));
                        Poll::Ready(Some(next))
                    }
                    Err(err) => Poll::Ready(Some(Err(err))),
                },
                None => Poll::Ready(None),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.source.size_hint()
        }
    }
}

#[cfg(feature = "async")]
use crate::async_operator::AsyncOperator;

/// Stream extension trait for indicators.
pub trait IndicatorStreamExt: Stream {
    /// Apply an [`Operator`] on the stream.
    fn indicator<P>(self, op: P) -> Operated<Self, P>
    where
        Self: Sized,
        P: Operator<Self::Item>,
    {
        Operated { source: self, op }
    }

    #[cfg(feature = "async")]
    /// Apply an [`AsyncOperator`] on the stream.
    fn async_indicator<P>(self, op: P) -> async_operated::Operated<Self, P>
    where
        Self: Sized,
        P: AsyncOperator<Self::Item>,
    {
        async_operated::Operated { source: self, op }
    }
}

impl<St: Stream> IndicatorStreamExt for St {}
