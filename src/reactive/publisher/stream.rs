use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use futures::{ready, Stream, TryStream};
use pin_project_lite::pin_project;

use crate::reactive::{Publisher, StreamError, Subscriber};

#[derive(Default)]
enum State {
    Feeding,
    Complete(Result<(), StreamError>),
    #[default]
    Closing,
}

pin_project! {
    /// Publisher drived by a stream.
    #[project = PublisherProj]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct StreamPublisher<'a, St, T> {
        subscriber: Option<Pin<Box<dyn Subscriber<T> + 'a>>>,
        #[pin]
        stream: St,
        buffered: Option<T>,
        state: State,
    }
}

impl<'a, St, T, E> Future for StreamPublisher<'a, St, T>
where
    St: TryStream<Ok = T, Error = E>,
    StreamError: From<E>,
{
    type Output = Result<(), StreamError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use State::*;

        let PublisherProj {
            subscriber,
            mut stream,
            buffered,
            state,
        } = self.project();
        let s = subscriber.as_mut().expect("publisher has finished");

        loop {
            match core::mem::take(state) {
                Feeding => {
                    *state = Feeding;
                    if buffered.is_some() {
                        let cancel = !ready!(s.as_mut().poll_ready(cx))?;
                        s.as_mut().start_send(buffered.take().unwrap())?;
                        if cancel {
                            *state = Complete(Err(StreamError::abort("cancelled")));
                        }
                    }
                    match stream.as_mut().try_poll_next(cx) {
                        Poll::Ready(Some(Ok(item))) => {
                            *buffered = Some(item);
                        }
                        Poll::Ready(Some(Err(err))) => {
                            *state = Complete(Err(err.into()));
                        }
                        Poll::Ready(None) => {
                            *state = Complete(Ok(()));
                        }
                        Poll::Pending => {
                            if !ready!(s.as_mut().poll_flush(cx))? {
                                *state = Complete(Err(StreamError::abort("cancelled")));
                            } else {
                                return Poll::Pending;
                            }
                        }
                    }
                }
                Complete(reason) => {
                    s.as_mut().closing(reason)?;
                }
                Closing => {
                    ready!(s.as_mut().poll_close(cx))?;
                    *subscriber = None;
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }
}

impl<'a, St, T, E> Publisher<'a> for StreamPublisher<'a, St, T>
where
    St: TryStream<Ok = T, Error = E> + 'a,
    StreamError: From<E>,
{
    type Output = T;

    fn subscribe<S>(&mut self, subscriber: S)
    where
        S: Subscriber<Self::Output> + 'a,
    {
        self.subscriber = Some(Box::pin(subscriber));
    }
}

/// Create a publisher from a stream.
pub fn stream<'a, T, E, St>(stream: St) -> StreamPublisher<'a, St, T>
where
    St: Stream<Item = Result<T, E>>,
    StreamError: From<E>,
{
    StreamPublisher {
        stream,
        subscriber: None,
        buffered: None,
        state: State::Feeding,
    }
}

#[cfg(test)]
mod tests {
    use crate::reactive::subscriber::unbounded;

    use super::*;
    use futures::{sink::unfold, stream::iter};
    use tracing::subscriber::DefaultGuard;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

    fn init_tracing() -> DefaultGuard {
        Registry::default()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .set_default()
    }

    #[tokio::test]
    async fn test_stream_publisher() -> anyhow::Result<()> {
        let _guard = init_tracing();
        let mut publisher = stream(iter([Ok(1), Ok(2), Ok(3), Ok(4)]));
        publisher.subscribe(unbounded(|res| {
            println!("{res:?}");
        }));
        publisher.await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_with_operator_processor() -> anyhow::Result<()> {
        use crate::{
            map,
            reactive::{processor::OperatorProcessor, PublisherExt},
        };
        let _guard = init_tracing();
        let mut publisher = stream(iter([
            Ok(1),
            Ok(2),
            Err(StreamError::unknown("error")),
            Ok(4),
        ]));
        let op1 = OperatorProcessor::new(map(|x| x + 1));
        let op2 = OperatorProcessor::new(map(|x| x * x));
        publisher.with(op1).with(op2).subscribe(unbounded(|res| {
            println!("{res:?}");
        }));
        publisher.await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_with_unfold() -> anyhow::Result<()> {
        use crate::{
            map,
            reactive::{
                processor::OperatorProcessor, subscriber::sink_with_shutdown, PublisherExt,
            },
        };
        let _guard = init_tracing();
        let mut publisher = stream(iter([
            Ok(1),
            Ok(2),
            Ok(3),
            Ok(4),
            Err(StreamError::unknown("error")),
            Ok(5),
        ]));
        let op1 = OperatorProcessor::new(map(|x| x + 1));
        let op2 = OperatorProcessor::new(map(|x| x * x));
        publisher.with(op1).with(op2).subscribe(sink_with_shutdown(
            unfold(0, |mut acc, item| async move {
                acc += item;
                println!("{acc}");
                Ok(acc)
            }),
            |res| {
                println!("{res:?}");
                Ok(())
            },
        ));
        publisher.await?;
        Ok(())
    }
}
