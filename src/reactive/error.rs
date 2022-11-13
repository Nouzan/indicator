#[cfg(feature = "std")]
use thiserror::Error;

/// Stream errors.
#[derive(Debug)]
#[cfg_attr(feature = "thiserror", derive(Error))]
pub enum StreamError {
    /// Abort or cancel.
    #[cfg_attr(feature = "thiserror", error("abort: {0}"))]
    Abort(String),
    /// Unknown.
    #[cfg_attr(feature = "thiserror", error("abort: {0}"))]
    Unknwon(String),
}

impl StreamError {
    /// Abort.
    pub fn abort(message: impl ToString) -> Self {
        Self::Abort(message.to_string())
    }

    /// Unknown.
    pub fn unknown(message: impl ToString) -> Self {
        Self::Unknwon(message.to_string())
    }
}
