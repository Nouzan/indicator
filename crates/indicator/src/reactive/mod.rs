/// Producer.
pub mod publisher;

/// Subscriber.
pub mod subscriber;

/// Processor.
pub mod processor;

/// Error.
pub mod error;

pub use self::{
    error::StreamError,
    processor::Processor,
    publisher::{Publisher, PublisherExt},
    subscriber::Subscriber,
};
