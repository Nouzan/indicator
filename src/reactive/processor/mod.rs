use super::{Publisher, Subscriber};

/// Processor.
pub trait Processor<I, O>: Subscriber<I> + Publisher<Output = O> {}

impl<I, O, P> Processor<I, O> for P where P: Subscriber<I> + Publisher<Output = O> {}
