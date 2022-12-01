use crate::reactive::StreamError;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::Sink;
use pin_project_lite::pin_project;

use super::Subscriber;

pin_project! {
    /// Sink Subscriber.
    pub struct SinkSubscriber<Si, F> {
        shutdown: Option<F>,
        #[pin]
        sink: Si,
    }
}

impl<I, E, Si, F> Subscriber<I> for SinkSubscriber<Si, F>
where
    Si: Sink<I, Error = E>,
    StreamError: From<E>,
    F: FnOnce(Result<(), StreamError>) -> Result<(), StreamError>,
{
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<bool, StreamError>> {
        self.project()
            .sink
            .poll_ready(cx)
            .map_err(StreamError::from)
            .map_ok(|_| true)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), StreamError> {
        self.project().sink.start_send(item)?;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<bool, StreamError>> {
        self.project()
            .sink
            .poll_flush(cx)
            .map_err(StreamError::from)
            .map_ok(|_| true)
    }

    fn closing(self: Pin<&mut Self>, reason: Result<(), StreamError>) -> Result<(), StreamError> {
        (self
            .project()
            .shutdown
            .take()
            .expect("`SinkSubscriber` has been closed"))(reason)?;
        Ok(())
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StreamError>> {
        self.project()
            .sink
            .poll_close(cx)
            .map_err(StreamError::from)
    }
}

/// Create a subscriber from a sink.
pub fn sink_with_shutdown<T, E, Si, F>(sink: Si, shutdown: F) -> SinkSubscriber<Si, F>
where
    Si: Sink<T, Error = E>,
    StreamError: From<E>,
    F: FnOnce(Result<(), StreamError>) -> Result<(), StreamError>,
{
    SinkSubscriber {
        shutdown: Some(shutdown),
        sink,
    }
}
