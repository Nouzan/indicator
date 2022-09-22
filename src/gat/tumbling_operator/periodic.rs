use crate::{prelude::GatOperator, Period, Tick, Tickable, TumblingWindow};

use super::{
    operator::{Operation, TumblingOperator},
    queue::{circular::Circular, Collection, Queue, QueueMut, QueueRef},
};

/// Periodic Operation.
pub trait Periodic<I, T> {
    /// Received an event from the same period.
    fn swap(&mut self, queue: QueueRef<T>, event: I) -> T;

    /// Received an event from a new period.
    fn push(&mut self, queue: QueueRef<T>, event: I) -> T;
}

/// Periodic `FnMut` wrapper.
#[derive(Debug, Clone, Copy)]
pub struct PeroidicFn<F>(F);

/// Create a [`PeroidicFn`].
pub fn periodic_fn<I, T, F>(f: F) -> impl Periodic<I, T>
where
    F: for<'a> FnMut(QueueRef<'a, T>, I, bool) -> T,
{
    PeroidicFn(f)
}

impl<I, T, F> Periodic<I, T> for PeroidicFn<F>
where
    F: for<'a> FnMut(QueueRef<'a, T>, I, bool) -> T,
{
    fn swap(&mut self, queue: QueueRef<T>, event: I) -> T {
        (self.0)(queue, event, false)
    }

    fn push(&mut self, queue: QueueRef<T>, event: I) -> T {
        (self.0)(queue, event, true)
    }
}

/// Identity periodic operation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Identity;

impl<I> Periodic<I, I> for Identity {
    fn swap(&mut self, _queue: QueueRef<I>, event: I) -> I {
        event
    }

    fn push(&mut self, _queue: QueueRef<I>, event: I) -> I {
        event
    }
}

/// Operation used in tumbling.
#[derive(Debug, Clone, Copy)]
pub struct Op<P> {
    last: Tick,
    period: Period,
    op: P,
}

impl<P> Op<P> {
    fn new(period: Period, op: P) -> Self {
        Self {
            last: Tick::BIG_BANG,
            period,
            op,
        }
    }
}

impl<I, T, P> Operation<I, T> for Op<P>
where
    I: Tickable,
    P: Periodic<I, T>,
{
    fn step(&mut self, mut queue: QueueMut<T>, event: I) {
        if self.period.same_window(&self.last, event.tick()) {
            let output = self.op.swap(queue.as_queue_ref(), event);
            queue.swap(output);
        } else {
            self.last = *event.tick();
            let output = self.op.push(queue.as_queue_ref(), event);
            queue.push(output);
        }
    }
}

/// Create a periodic operator.
pub fn periodic_with<Q, I, P>(queue: Q, period: Period, op: P) -> TumblingOperator<Q, Op<P>>
where
    I: Tickable,
    P: Periodic<I, Q::Item>,
    Q: Queue,
{
    TumblingOperator::with_queue(queue, Op::new(period, op))
}

/// Create a new periodic operator with circular queue.
pub fn periodic<const N: usize, I, O, P>(
    length: usize,
    period: Period,
    op: P,
) -> TumblingOperator<Circular<O, N>, Op<P>>
where
    I: Tickable,
    P: Periodic<I, O>,
{
    TumblingOperator::with_queue(Circular::with_capacity(length), Op::new(period, op))
}

/// Create a cache operator.
pub fn cache<const N: usize, I>(
    length: usize,
    period: Period,
) -> impl for<'out> GatOperator<I, Output<'out> = QueueRef<'out, I>>
where
    I: Tickable + 'static,
{
    periodic::<N, _, _, _>(length, period, Identity)
}
