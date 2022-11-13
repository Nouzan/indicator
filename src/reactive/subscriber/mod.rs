use super::error::StreamError;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

pub use self::{sink::sink_with_shutdown, unbounded::unbounded};

/// Unbounded Subscriber.
pub mod unbounded;

/// Sink Subscriber.
pub mod sink;

/// Subscriber.
pub trait Subscriber<I> {
    /// Poll ready.
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<bool, StreamError>>;
    /// Feed next item.
    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), StreamError>;
    /// Poll flush.
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<bool, StreamError>>;
    /// Closing.
    fn closing(self: Pin<&mut Self>, reason: Result<(), StreamError>) -> Result<(), StreamError>;
    /// Poll close.
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), StreamError>>;
}
