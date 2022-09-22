use crate::gat::GatOperator;

use super::queue::{circular::Circular, Collection, Queue, QueueRef, Tumbling};

/// Operation.
pub trait Operation<I, Q: Queue> {
    /// Step.
    fn step(&mut self, w: &mut Tumbling<Q>, x: I);
}

impl<I, Q, F> Operation<I, Q> for F
where
    Q: Queue,
    F: FnMut(&mut Tumbling<Q>, I),
{
    #[inline]
    fn step(&mut self, w: &mut Tumbling<Q>, x: I) {
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
        P: Operation<I, Q>,
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
    P: Operation<I, Q>,
{
    type Output<'out> = QueueRef<'out, Q::Item>
    where
        Self: 'out,
        I: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        let Self { queue, op } = self;
        op.step(queue, input);
        queue.as_queue_ref()
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

/// View Operator.
#[derive(Debug, Clone, Copy)]
pub struct ViewOperator<F> {
    f: F,
}

impl<'a, I, O, F> GatOperator<QueueRef<'a, I>> for ViewOperator<F>
where
    F: for<'input> FnMut(QueueRef<'input, I>) -> O,
{
    type Output<'out> = O
    where
        Self: 'out,
        'a: 'out;

    fn next<'out>(&'out mut self, input: QueueRef<'a, I>) -> Self::Output<'out>
    where
        'a: 'out,
    {
        (self.f)(input)
    }
}

/// Create a new view operator from a closure.
pub fn view<I, O, F>(f: F) -> ViewOperator<F>
where
    F: for<'input> FnMut(QueueRef<'input, I>) -> O,
{
    ViewOperator { f }
}
