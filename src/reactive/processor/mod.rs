pub use self::operator::OperatorProcessor;
use super::{Publisher, Subscriber};

/// Operator Processor.
pub mod operator;

/// Processor.
pub trait Processor<'a, I>: Subscriber<I> + Publisher<'a> {}

impl<'a, I, P> Processor<'a, I> for P where P: Subscriber<I> + Publisher<'a> {}
