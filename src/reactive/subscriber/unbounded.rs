use core::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::reactive::StreamError;

use super::Subscriber;

pin_project! {
    /// A unbounded subscriber.
    pub struct Unbounded<F> {
        f: F,
    }
}

impl<I, F> Subscriber<I> for Unbounded<F>
where
    F: FnMut(Result<I, StreamError>),
{
    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<bool, StreamError>> {
        Poll::Ready(Ok(true))
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), StreamError> {
        (self.project().f)(Ok(item));
        Ok(())
    }

    fn closing(self: Pin<&mut Self>, reason: Result<(), StreamError>) -> Result<(), StreamError> {
        if let Err(err) = reason {
            (self.project().f)(Err(err));
        }
        Ok(())
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), StreamError>> {
        Poll::Ready(Ok(()))
    }

    fn poll_flush(self: Pin<&mut Self>) -> Poll<Result<bool, StreamError>> {
        Poll::Ready(Ok(true))
    }
}

/// Create a unbounded subscriber.
pub fn unbounded<I, F>(f: F) -> Unbounded<F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    Unbounded { f }
}
