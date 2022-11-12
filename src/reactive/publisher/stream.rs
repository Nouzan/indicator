use core::{future::IntoFuture, num::NonZeroUsize};
use futures::{future::BoxFuture, FutureExt, Stream, TryStreamExt};
use std::sync::{Arc, RwLock};
use tokio::sync::{oneshot, Semaphore};

use crate::reactive::{Publisher, StreamError, Subscriber, Subscription};

/// Publisher drived by a stream.
pub struct StreamPublisher<'a, St, T> {
    subscriber: Option<Box<dyn Subscriber<T> + 'a>>,
    stream: St,
}

struct StreamSubscription {
    semaphore: Arc<Semaphore>,
    err_tx: RwLock<Option<oneshot::Sender<StreamError>>>,
    #[allow(dead_code)]
    cancel_rx: oneshot::Receiver<()>,
}

impl Subscription for StreamSubscription {
    fn request(&self, num: NonZeroUsize) {
        if self.err_tx.read().expect("lock `err_tx` error").is_none() {
            return;
        }
        if self.semaphore.available_permits() + num.get() > usize::MAX >> 3 {
            _ = self
                .err_tx
                .write()
                .expect("lock `err_tx` error")
                .take()
                .expect("`err_tx` must exist")
                .send(StreamError::abort("too many permits"));
        } else {
            self.semaphore.add_permits(num.get());
        }
    }

    fn unbounded(&self) {
        self.semaphore.close();
    }
}

async fn process<'a, T, E>(
    stream: impl Stream<Item = Result<T, E>>,
    subscriber: &mut Box<dyn Subscriber<T> + 'a>,
    err_tx: oneshot::Sender<StreamError>,
) -> Result<(), StreamError>
where
    StreamError: From<E>,
{
    let semaphore = Arc::new(Semaphore::new(0));
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let subscription = StreamSubscription {
        semaphore: semaphore.clone(),
        err_tx: RwLock::new(Some(err_tx)),
        cancel_rx,
    };
    subscriber.on_subscribe(subscription.boxed());
    futures::pin_mut!(stream);
    let mut unbounded = false;
    while let Some(item) = stream.try_next().await? {
        if unbounded {
            subscriber.on_next(item);
        } else {
            match semaphore.acquire().await {
                Ok(permit) => {
                    permit.forget();
                    subscriber.on_next(item)
                }
                Err(_) => {
                    if cancel_tx.is_closed() {
                        break;
                    } else {
                        subscriber.on_next(item);
                        unbounded = true;
                    }
                }
            }
        }
    }
    Ok(())
}

impl<'a, St, T, E> IntoFuture for StreamPublisher<'a, St, T>
where
    T: Send + 'a,
    E: Send + 'a,
    St: Stream<Item = Result<T, E>> + Send + 'a,
    StreamError: From<E>,
{
    type Output = ();

    type IntoFuture = BoxFuture<'a, ()>;

    fn into_future(self) -> Self::IntoFuture {
        self.run().boxed()
    }
}

impl<'a, St, T, E> StreamPublisher<'a, St, T>
where
    St: Stream<Item = Result<T, E>>,
    StreamError: From<E>,
{
    /// Run the publisher.
    pub async fn run(self) {
        let Self { subscriber, stream } = self;
        if let Some(mut subscriber) = subscriber {
            let (err_tx, err_rx) = oneshot::channel();
            tokio::select! {
                res = err_rx => {
                    match res {
                        Ok(err) => subscriber.on_error(err).await,
                        Err(_) => subscriber.on_complete().await,
                    }
                }
                res = process(stream, &mut subscriber, err_tx) => {
                    match res {
                        Ok(()) => subscriber.on_complete().await,
                        Err(err) => subscriber.on_error(err).await,
                    }
                }
            }
        }
    }
}

impl<'a, St, T, E> Publisher<'a> for StreamPublisher<'a, St, T>
where
    St: Stream<Item = Result<T, E>> + Send + 'a,
    T: Send,
    E: Send,
    StreamError: From<E>,
{
    type Output = T;

    fn subscribe<S>(&mut self, subscriber: S)
    where
        S: Subscriber<Self::Output> + 'a,
    {
        self.subscriber = Some(Box::new(subscriber));
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
        let mut publisher = stream(iter([Ok(1), Ok(2), Ok(3), Ok(4)]));
        publisher.subscribe(unbounded(|res| {
            println!("{res:?}");
        }));
        publisher.await;
        Ok(())
    }

    #[cfg(feature = "operator-processor")]
    #[tokio::test]
    async fn test_with_operator_processor() -> anyhow::Result<()> {
        use crate::{
            map,
            reactive::{processor::OperatorProcessor, PublisherExt},
        };

        let mut publisher = stream(iter([Ok(1), Ok(2), Ok(3), Ok(4)]));
        let op1 = OperatorProcessor::new(map(|x| x + 1));
        let op2 = OperatorProcessor::new(map(|x| x * x));
        publisher.with(op1).with(op2).subscribe(unbounded(|res| {
            println!("{res:?}");
        }));
        publisher.await;
        Ok(())
    }

    #[cfg(all(feature = "operator-processor", feature = "task-subscriber"))]
    #[tokio::test]
    async fn test_with_task_subscriber() -> anyhow::Result<()> {
        use core::future::ready;

        use crate::{
            map,
            reactive::{processor::OperatorProcessor, subscriber::subscriber_fn, PublisherExt},
        };

        let _guard = init_tracing();
        let mut publisher = stream(iter([Ok(1), Ok(2), Ok(3), Ok(4), Ok(5), Ok(6)]));
        let op1 = OperatorProcessor::new(map(|x| x + 1));
        let op2 = OperatorProcessor::new(map(|x| x * x));
        let mut count = 0;
        publisher
            .with(op1)
            .with(op2)
            .subscribe(subscriber_fn(move |data| {
                if count >= 4 {
                    ready(false).left_future()
                } else {
                    count += 1;
                    async move {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        println!("count={count}, data={data:?}");
                        count < 4
                    }
                    .right_future()
                }
            }));
        publisher.await;
        Ok(())
    }
}
