use core::future::IntoFuture;
use tokio::sync::{mpsc, oneshot};

use crate::reactive::{BoxSubscription, StreamError};

use super::Subscriber;

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
    }
}

/// Task subscriber.
#[derive(Debug)]
pub struct Task<T, F> {
    next: Option<F>,
    tx: Option<mpsc::UnboundedSender<Result<T, StreamError>>>,
    cancel_rx: Option<oneshot::Receiver<()>>,
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
            tokio::spawn(async move {
                tokio::select! {
                    _ = run => {
                    println!("subscription cancelled");
                    },
                    _ = cancel_tx.closed() => {
                    println!("subscription completed");
                    }
                }
            });
        }
    }

    fn on_next(&mut self, input: T) {
        if let Some(tx) = self.tx.as_ref() {
            _ = tx.send(Ok(input));
        }
    }

    fn on_error(&mut self, error: StreamError) {
        let (Some(tx), Some(mut rx)) = (self.tx.take(), self.cancel_rx.take()) else {
	    panic!("`tx` and `cancel_rx` must exist here");
	};
        _ = tx.send(Err(error));
        rx.close();
    }

    fn on_complete(&mut self) {
        let (Some(_tx), Some(mut rx)) = (self.tx.take(), self.cancel_rx.take()) else {
	    panic!("`tx` and `cancel_rx` must exist here");
	};
        rx.close();
    }
}
