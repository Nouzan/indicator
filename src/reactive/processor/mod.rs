use super::{Publisher, Subscriber};

/// Operator Processor.
#[cfg(feature = "operator-processor")]
pub mod operator;

/// Processor.
pub trait Processor<'a, I, O>: Subscriber<I> + Publisher<'a, Output = O> {}

impl<'a, I, O, P> Processor<'a, I, O> for P where P: Subscriber<I> + Publisher<'a, Output = O> {}
