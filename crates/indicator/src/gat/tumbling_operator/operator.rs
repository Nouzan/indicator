use core::num::NonZeroUsize;

use crate::gat::GatOperator;

use super::queue::{circular::Circular, Collection, Queue, QueueMut, Tumbling};

/// Operation.
pub trait Operation<I, T> {
    /// Output.
    type Output<'out>
    where
        T: 'out;

    /// Step.
    fn step<'a>(&mut self, w: QueueMut<'a, T>, x: I) -> Self::Output<'a>;
}

impl<I, T, F> Operation<I, T> for F
where
    F: for<'a> FnMut(QueueMut<'a, T>, I),
{
    type Output<'out> = () where T: 'out;

    #[inline]
    fn step(&mut self, w: QueueMut<T>, x: I) {
        (self)(w, x)
    }
}

/// Tumbling Operator.
pub struct TumblingOperator<Q: Queue, P> {
    queue: Tumbling<Q>,
    op: P,
}

impl<Q: Queue, P> TumblingOperator<Q, P> {
    /// Create a new tumbling operator.
    pub fn with_queue<I>(queue: Q, op: P) -> Self
    where
        Q: Queue,
        P: Operation<I, Q::Item>,
    {
        Self {
            op,
            queue: Tumbling::new(queue),
        }
    }
}

impl<I, Q, P> GatOperator<I> for TumblingOperator<Q, P>
where
    Q: Queue,
    P: Operation<I, Q::Item>,
{
    type Output<'out> = P::Output<'out>
    where
        Self: 'out,
        I: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        let Self { queue, op } = self;
        op.step(queue.as_queue_mut(), input)
    }
}

/// Create a new tumbling operator with circular queue.
pub fn tumbling<const N: usize, I, T, P>(
    length: NonZeroUsize,
    op: P,
) -> TumblingOperator<Circular<N, T>, P>
where
    P: for<'a> FnMut(QueueMut<'a, T>, I),
{
    TumblingOperator::with_queue(Circular::with_capacity(length.get()), op)
}
