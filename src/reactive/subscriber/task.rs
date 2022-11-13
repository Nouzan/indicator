use core::{future::IntoFuture, pin::Pin, task::{Poll, Context}};
use futures::FutureExt;
use tokio::sync::{mpsc, oneshot};

use crate::reactive::{BoxSubscription, StreamError};

use super::{Subscriber};

/// Create a [`Subscriber`] from the given function.
pub fn subscriber_fn<T, F, Fut>(f: F) -> Task<T, F>
where
    T: Send + 'static,
    F: FnMut(Result<T, StreamError>) -> Fut + Send + 'static,
    Fut: IntoFuture<Output = bool> + Send,
    Fut::IntoFuture: Send,
{
    Task {
        next: Some(f),
        tx: None,
        cancel_rx: None,
        handle: None,
    }
}

/// Task subscriber.
#[derive(Debug)]
pub struct Task<T, F> {
    next: Option<F>,
    tx: Option<mpsc::UnboundedSender<Result<T, StreamError>>>,
    cancel_rx: Option<oneshot::Receiver<()>>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl<T, F, Fut> Subscriber<T> for Task<T, F>
where
    T: Send + 'static,
    F: FnMut(Result<T, StreamError>) -> Fut + Send + 'static,
    Fut: IntoFuture<Output = bool> + Send,
    Fut::IntoFuture: Send,
{
    fn on_subscribe(&mut self, subscription: BoxSubscription) {
        if let Some(mut next) = self.next.take() {
            let (tx, mut rx) = mpsc::unbounded_channel();
            let (mut cancel_tx, cancel_rx) = oneshot::channel();
            self.tx = Some(tx);
            self.cancel_rx = Some(cancel_rx);
            let run = async move {
                let mut subscription = Some(subscription);
                loop {
                    if let Some(s) = subscription.as_mut() {
                        s.request(1.try_into().unwrap());
                    }
                    let Some(data) = rx.recv().await else {
			break;
		    };
                    if !(next)(data).await {
                        subscription.take();
                    }
                }
            };
            let handle = tokio::spawn(async move {
                tokio::select! {
                    _ = run => {
                    tracing::trace!("subscription cancelled");
                    },
                    _ = cancel_tx.closed() => {
                    tracing::trace!("subscription completed");
                    }
                }
            });
            self.handle = Some(handle);
        }
    }

    fn on_next(&mut self, input: T) {
        if let Some(tx) = self.tx.as_ref() {
            _ = tx.send(Ok(input));
        }
    }

    fn on_error(&mut self, error: StreamError) -> Complete<'_> {
        let (Some(tx), Some(mut rx), Some(handle)) = (self.tx.take(), self.cancel_rx.take(), self.handle.take()) else {
	    panic!("they all must exist here");
	};
        _ = tx.send(Err(error));
        rx.close();
        handle
            .map(|res| {
                if let Err(err) = res {
                    tracing::error!(%err, "task error");
                }
            })
            .boxed()
    }

    fn on_complete(&mut self) -> Complete<'_> {
        let (Some(_tx), Some(mut rx), Some(handle)) = (self.tx.take(), self.cancel_rx.take(), self.handle.take()) else {
	    panic!("they all must exist here");
	};
        rx.close();
        handle
            .map(|res| {
                if let Err(err) = res {
                    tracing::error!(%err, "task error");
                }
            })
            .boxed()
    }

    fn poll_ready(self: Pin<&mut Self>, cx: Context<'_>) -> Poll<Result<(), StreamError>> {
        todo!()
    }

    fn feed_next(self: Pin<&mut Self>, item: T) -> Result<(), StreamError> {
        todo!()
    }

    fn poll_finish(
        self: Pin<&mut Self>,
        cx: Context<'_>,
        reason: Result<(), StreamError>,
    ) -> Poll<Result<(), StreamError>> {
        todo!()
    }
}
