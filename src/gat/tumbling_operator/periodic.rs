use crate::{prelude::GatOperator, Period, Tick, Tickable, TumblingWindow};

use super::{
    operator::{Operation, TumblingOperator},
    queue::{circular::Circular, Queue, Tumbling},
};

/// Periodic Operation.
pub trait Periodic<I, Q>
where
    Q: Queue<Item = Self::Output>,
{
    /// The output type.
    type Output;

    /// Received an event from the same period.
    fn swap(&mut self, queue: &Tumbling<Q>, event: I) -> Self::Output;

    /// Received an event from a new period.
    fn push(&mut self, queue: &Tumbling<Q>, event: I) -> Self::Output;
}

/// Identity periodic operation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Identity;

impl<I, Q> Periodic<I, Q> for Identity
where
    Q: Queue<Item = I>,
{
    type Output = I;

    fn swap(&mut self, _queue: &Tumbling<Q>, event: I) -> Self::Output {
        event
    }

    fn push(&mut self, _queue: &Tumbling<Q>, event: I) -> Self::Output {
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

impl<I, Q, P> Operation<I, Q> for Op<P>
where
    I: Tickable,
    P: Periodic<I, Q>,
    Q: Queue<Item = P::Output>,
{
    fn step(&mut self, queue: &mut Tumbling<Q>, event: I) {
        if self.period.same_window(&self.last, event.tick()) {
            let output = self.op.swap(queue, event);
            queue.swap(output);
        } else {
            self.last = *event.tick();
            let output = self.op.push(queue, event);
            queue.push(output);
        }
    }
}

/// Create a periodic operator.
pub fn periodic_with<Q, I, P>(length: usize, period: Period, op: P) -> TumblingOperator<Q, Op<P>>
where
    I: Tickable,
    P: Periodic<I, Q>,
    Q: Queue<Item = P::Output>,
{
    TumblingOperator::with_queue(Q::with_capacity(length), Op::new(period, op))
}

/// Create a new periodic operator with circular queue.
pub fn periodic<const N: usize, I, O, P>(
    length: usize,
    period: Period,
    op: P,
) -> TumblingOperator<Circular<O, N>, Op<P>>
where
    I: Tickable,
    P: Periodic<I, Circular<O, N>, Output = O>,
{
    TumblingOperator::with_queue(Circular::with_capacity(length), Op::new(period, op))
}

/// Create a cache operator.
pub fn cache<const N: usize, I>(
    length: usize,
    period: Period,
) -> impl for<'out> GatOperator<I, Output<'out> = &'out Tumbling<Circular<I, N>>>
where
    I: Tickable + 'static,
{
    periodic(length, period, Identity)
}
