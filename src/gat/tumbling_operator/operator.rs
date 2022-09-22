use crate::gat::GatOperator;

use super::queue::{circular::Circular, Collection, Queue, QueueMut, QueueRef, Tumbling};

/// Operation.
pub trait Operation<I, T> {
    /// Step.
    fn step(&mut self, w: QueueMut<T>, x: I);
}

impl<I, T, F> Operation<I, T> for F
where
    F: for<'a> FnMut(QueueMut<'a, T>, I),
{
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
    type Output<'out> = QueueRef<'out, Q::Item>
    where
        Self: 'out,
        I: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        let Self { queue, op } = self;
        op.step(queue.as_queue_mut(), input);
        queue.as_queue_ref()
    }
}

/// Create a new tumbling operator with circular queue.
pub fn tumbling<const N: usize, I, T, P>(
    length: usize,
    op: P,
) -> TumblingOperator<Circular<T, N>, P>
where
    P: for<'a> FnMut(QueueMut<'a, T>, I),
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
