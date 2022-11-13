use core::{
    pin::Pin,
    task::{Context, Poll},
};

use pin_project_lite::pin_project;

use crate::reactive::{StreamError};

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
    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<bool> {
        Poll::Ready(true)
    }

    fn start_send(self: Pin<&mut Self>, item: I) {
        (self.project().f)(Ok(item));
    }

    fn closing(self: Pin<&mut Self>, reason: Result<(), StreamError>) {
        if let Err(err) = reason {
            (self.project().f)(Err(err));
        }
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        Poll::Ready(())
    }

    fn poll_flush(self: Pin<&mut Self>) -> Poll<bool> {
        Poll::Ready(true)
    }
}

/// Create a unbounded subscriber.
pub fn unbounded<I, F>(f: F) -> Unbounded<F>
where
    F: FnMut(Result<I, StreamError>) + Send,
{
    Unbounded { f }
}
