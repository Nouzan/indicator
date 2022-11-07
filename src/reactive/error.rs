#[cfg(feature = "std")]
use thiserror::Error;

/// Stream errors.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum StreamError {
    /// Abort.
    #[cfg_attr(feature = "std", error("abort: {0}"))]
    Abort(String),
}

impl StreamError {
    /// Abort.
    pub fn abort(message: impl ToString) -> Self {
        Self::Abort(message.to_string())
    }
}
