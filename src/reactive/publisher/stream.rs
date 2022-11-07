use core::num::NonZeroUsize;
use futures::{future::BoxFuture, FutureExt, Stream, TryStreamExt};
use std::sync::Arc;
use tokio::sync::{oneshot, Semaphore};

use crate::reactive::{Publisher, StreamError, Subscriber, Subscription};

/// Publisher drived by a stream.
#[derive(Debug, Clone, Copy)]
pub struct StreamPublisher<St> {
    stream: St,
}

struct StreamSubscription {
    semaphore: Arc<Semaphore>,
    err_tx: Option<oneshot::Sender<StreamError>>,
    #[allow(dead_code)]
    cancel_rx: oneshot::Receiver<()>,
}

impl Subscription for StreamSubscription {
    fn request(&mut self, num: NonZeroUsize) {
        if self.semaphore.available_permits() + num.get() > usize::MAX >> 3 {
            self.err_tx
                .take()
                .expect("`err_tx` must exist")
                .send(StreamError::abort("too many permits"))
                .expect("`err_tx` send must success");
        } else {
            self.semaphore.add_permits(num.get());
        }
    }

    fn unbounded(&mut self) {
        self.semaphore.close();
    }
}

async fn process<'a, T, E>(
    stream: impl Stream<Item = Result<T, E>>,
    subscriber: &mut impl Subscriber<'a, T>,
    err_tx: oneshot::Sender<StreamError>,
) -> Result<(), StreamError>
where
    StreamError: From<E>,
{
    let semaphore = Arc::new(Semaphore::new(0));
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let subscription = StreamSubscription {
        semaphore: semaphore.clone(),
        err_tx: Some(err_tx),
        cancel_rx,
    };
    subscriber.on_subscribe(subscription);
    futures::pin_mut!(stream);
    let mut unbounded = false;
    while let Some(item) = stream.try_next().await? {
        if unbounded {
            subscriber.on_next(item);
        } else {
            match semaphore.acquire().await {
                Ok(_permit) => subscriber.on_next(item),
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

impl<St, T, E> Publisher for StreamPublisher<St>
where
    St: Stream<Item = Result<T, E>> + Send,
    T: Send,
    E: Send,
    StreamError: From<E>,
{
    type Output = T;
    type Task<'a> = BoxFuture<'a, ()> where St: 'a;

    fn subscribe<'a, S>(self, mut subscriber: S) -> Self::Task<'a>
    where
        St: 'a,
        S: Subscriber<'a, Self::Output> + 'a,
    {
        let stream = self.stream;
        async move {
            let (err_tx, err_rx) = oneshot::channel();
            tokio::select! {
                res = err_rx => {
                    match res {
                        Ok(err) => subscriber.on_error(err),
                        Err(_) => subscriber.on_complete(),
                    }
                }
                res = process(stream, &mut subscriber, err_tx) => {
                    match res {
                        Ok(()) => subscriber.on_complete(),
                        Err(err) => subscriber.on_error(err),
                    }
                }
            }
        }
        .boxed()
    }
}

/// Create a publisher from a stream.
pub fn stream<T, E, St>(stream: St) -> StreamPublisher<St>
where
    St: Stream<Item = Result<T, E>> + Send,
    T: Send,
    E: Send,
    StreamError: From<E>,
{
    StreamPublisher { stream }
}

#[cfg(test)]
mod tests {
    use crate::reactive::subscriber::unbounded;

    use super::*;
    use futures::stream::iter;

    #[tokio::test]
    async fn test_stream_publisher() -> anyhow::Result<()> {
        let publisher = stream(iter([Ok(1), Ok(2), Ok(3), Ok(4)]));
        publisher
            .subscribe(unbounded(|res| {
                println!("{res:?}");
            }))
            .await;
        Ok(())
    }
}
