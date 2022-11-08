use super::{Publisher, Subscriber};

/// Processor.
pub trait Processor<'a, I, O>: Subscriber<'a, I> + Publisher<Output = O> {}

impl<'a, I, O, P> Processor<'a, I, O> for P where P: Subscriber<'a, I> + Publisher<Output = O> {}
