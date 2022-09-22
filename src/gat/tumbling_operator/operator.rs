use crate::gat::GatOperator;

use super::queue::{circular::Circular, Queue, Tumbling};

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
        P: FnMut(&mut Tumbling<Q>, I),
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
    P: FnMut(&mut Tumbling<Q>, I),
{
    type Output<'out> = &'out Tumbling<Q>
    where
        Self: 'out,
        I: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        let Self { queue, op } = self;
        (op)(queue, input);
        queue
    }
}

/// Create a new tumbling operator with circular queue.
pub fn tumbling<const N: usize, I, T, P>(
    length: usize,
    op: P,
) -> TumblingOperator<Circular<T, N>, P>
where
    P: FnMut(&mut Tumbling<Circular<T, N>>, I),
{
    TumblingOperator::with_queue(Circular::with_capacity(length), op)
}
