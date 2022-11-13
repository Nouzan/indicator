use super::{Publisher, Subscriber};

// /// Operator Processor.
// #[cfg(feature = "operator-processor")]
// pub mod operator;

// #[cfg(feature = "operator-processor")]
// pub use self::operator::OperatorProcessor;

/// Processor.
pub trait Processor<'a, I>: Subscriber<I> + Publisher<'a> {}

impl<'a, I, P> Processor<'a, I> for P where P: Subscriber<I> + Publisher<'a> {}
