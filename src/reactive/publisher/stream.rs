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
            match std::mem::take(state) {
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
                            if !ready!(s.as_mut().poll_flush())? {
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
    St: Stream<Item = Result<T, E>> + Send,
    T: Send,
    E: Send,
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
    use futures::stream::iter;
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

    // #[cfg(feature = "operator-processor")]
    // #[tokio::test]
    // async fn test_with_operator_processor() -> anyhow::Result<()> {
    //     use crate::{
    //         map,
    //         reactive::{processor::OperatorProcessor, PublisherExt},
    //     };

    //     let mut publisher = stream(iter([Ok(1), Ok(2), Ok(3), Ok(4)]));
    //     let op1 = OperatorProcessor::new(map(|x| x + 1));
    //     let op2 = OperatorProcessor::new(map(|x| x * x));
    //     publisher.with(op1).with(op2).subscribe(unbounded(|res| {
    //         println!("{res:?}");
    //     }));
    //     publisher.await;
    //     Ok(())
    // }

    // #[cfg(all(feature = "operator-processor", feature = "task-subscriber"))]
    // #[tokio::test]
    // async fn test_with_task_subscriber() -> anyhow::Result<()> {
    //     use core::future::ready;

    //     use crate::{
    //         map,
    //         reactive::{processor::OperatorProcessor, subscriber::subscriber_fn, PublisherExt},
    //     };

    //     let _guard = init_tracing();
    //     let mut publisher = stream(iter([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5), Ok(6)]));
    //     let op1 = OperatorProcessor::new(map(|x| x + 1));
    //     let op2 = OperatorProcessor::new(map(|x| x * x));
    //     let mut count = 0;
    //     publisher
    //         .with(op1)
    //         .with(op2)
    //         .subscribe(subscriber_fn(move |data| {
    //             if count >= 4 {
    //                 ready(false).left_future()
    //             } else {
    //                 count += 1;
    //                 async move {
    //                     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    //                     println!("count={count}, data={data:?}");
    //                     count < 4
    //                 }
    //                 .right_future()
    //             }
    //         }));
    //     publisher.await;
    //     Ok(())
    // }
}
