use crate::{prelude::GatOperator, Period, Tick, Tickable, TumblingWindow};

use super::{
    operator::{Operation, TumblingOperator},
    queue::{circular::Circular, Collection, Queue, QueueMut, QueueRef},
};

/// Periodic Operation.
pub trait PeriodicOp<I, T> {
    /// Received an event from the same period.
    fn swap(&mut self, queue: QueueRef<T>, event: I) -> T;

    /// Received an event from a new period.
    fn push(&mut self, queue: QueueRef<T>, event: I) -> T;
}

/// Periodic `FnMut` wrapper.
#[derive(Debug, Clone, Copy)]
pub struct PeroidicFn<F>(F);

impl<I, T, F> PeriodicOp<I, T> for PeroidicFn<F>
where
    F: for<'a> FnMut(QueueRef<'a, T>, bool, I) -> T,
{
    fn swap(&mut self, queue: QueueRef<T>, event: I) -> T {
        (self.0)(queue, false, event)
    }

    fn push(&mut self, queue: QueueRef<T>, event: I) -> T {
        (self.0)(queue, true, event)
    }
}

/// Identity periodic operation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Identity;

impl<I> PeriodicOp<I, I> for Identity {
    fn swap(&mut self, _queue: QueueRef<I>, event: I) -> I {
        event
    }

    fn push(&mut self, _queue: QueueRef<I>, event: I) -> I {
        event
    }
}

/// Operation used in tumbling.
#[derive(Debug, Clone, Copy)]
pub struct Op<P, const PUSH_FIRST: bool> {
    last: Tick,
    period: Period,
    op: P,
}

impl<P, const PUSH_FIRST: bool> Op<P, PUSH_FIRST> {
    fn new(period: Period, op: P) -> Self {
        Self {
            last: Tick::BIG_BANG,
            period,
            op,
        }
    }
}

impl<I, T, P> Operation<I, T> for Op<P, false>
where
    I: Tickable,
    P: PeriodicOp<I, T>,
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

impl<I, T, P> Operation<I, T> for Op<P, true>
where
    I: Tickable,
    T: Clone,
    P: PeriodicOp<I, T>,
{
    fn step(&mut self, mut queue: QueueMut<T>, event: I) {
        if self.period.same_window(&self.last, event.tick()) {
            let output = self.op.swap(queue.as_queue_ref(), event);
            queue.swap(output);
        } else {
            self.last = *event.tick();
            if let Some(last) = queue.get(0).cloned() {
                queue.push(last);
                let mut output = self.op.push(queue.as_queue_ref(), event);
                let last = queue.get_mut(0).unwrap();
                std::mem::swap(last, &mut output);
            } else {
                let output = self.op.push(queue.as_queue_ref(), event);
                queue.push(output);
            }
        }
    }
}

/// Periodic Operator Builder.
#[derive(Debug, Clone, Copy)]
pub struct Periodic<Q, const PUSH_FIRST: bool> {
    queue: Q,
    period: Period,
}

impl<Q, const PUSH_FIRST: bool> Periodic<Q, PUSH_FIRST>
where
    Q: Queue,
{
    /// Create a new periodic operator with the given queue.
    pub fn new(queue: Q, period: Period) -> Self {
        Self { queue, period }
    }
}

impl<Q> Periodic<Q, true>
where
    Q: Queue,
    Q::Item: Clone,
{
    /// Build the periodic operator.
    pub fn build<I, P>(self, op: P) -> TumblingOperator<Q, Op<P, true>>
    where
        I: Tickable,
        P: PeriodicOp<I, Q::Item>,
    {
        TumblingOperator::with_queue(self.queue, Op::new(self.period, op))
    }

    /// Build the periodic operator using the given closure.
    pub fn build_fn<I, F>(self, f: F) -> TumblingOperator<Q, Op<PeroidicFn<F>, true>>
    where
        I: Tickable,
        F: for<'a> FnMut(QueueRef<'a, Q::Item>, bool, I) -> Q::Item,
    {
        self.build(PeroidicFn(f))
    }
}

impl<Q> Periodic<Q, false>
where
    Q: Queue,
{
    /// Build periodic operator.
    pub fn build<I, P>(self, op: P) -> TumblingOperator<Q, Op<P, false>>
    where
        I: Tickable,
        P: PeriodicOp<I, Q::Item>,
    {
        TumblingOperator::with_queue(self.queue, Op::new(self.period, op))
    }

    /// Build a cache operator.
    pub fn build_cache(
        self,
    ) -> impl for<'out> GatOperator<Q::Item, Output<'out> = QueueRef<'out, Q::Item>>
    where
        Q: Queue + 'static,
        Q::Item: Tickable + 'static,
    {
        self.build(Identity)
    }

    /// Build the periodic operator using the given closure.
    pub fn build_fn<I, F>(self, f: F) -> TumblingOperator<Q, Op<PeroidicFn<F>, false>>
    where
        I: Tickable,
        F: for<'a> FnMut(QueueRef<'a, Q::Item>, bool, I) -> Q::Item,
    {
        self.build(PeroidicFn(f))
    }

    /// Push before calculation.
    pub fn push_first(self) -> Periodic<Q, true>
    where
        Q::Item: Clone,
    {
        Periodic::new(self.queue, self.period)
    }
}

impl Periodic<(), false> {
    /// Create a new periodic operator builder using circular queue.
    pub fn with_circular<T>(length: usize, period: Period) -> Periodic<Circular<T, 0>, false> {
        Periodic::new(Circular::with_capacity(length), period)
    }

    /// Create a new periodic operator builder using circular queue.
    pub fn with_circular_n<const N: usize, T>(period: Period) -> Periodic<Circular<T, N>, false> {
        Periodic::new(Circular::with_capacity(N), period)
    }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use crate::prelude::*;

    #[test]
    fn push_first() {
        let mut cache = 0;
        let mut op = Periodic::with_circular_n::<2, TickValue<usize>>(Period::secs(2))
            .push_first()
            .build_fn(|w, n, x: TickValue<usize>| {
                if n && w.len() > 1 {
                    assert!(w[0] == w[1]);
                    cache = w[1].value;
                } else if w.len() > 1 {
                    assert!(w[1].value == cache);
                }
                x
            });

        for x in [
            TickValue::new(datetime!(2022-09-23 00:00:00 +00:00), 1),
            TickValue::new(datetime!(2022-09-23 00:00:01 +00:00), 2),
            TickValue::new(datetime!(2022-09-23 00:00:02 +00:00), 3),
            TickValue::new(datetime!(2022-09-23 00:00:03 +00:00), 4),
            TickValue::new(datetime!(2022-09-23 00:00:04 +00:00), 5),
            TickValue::new(datetime!(2022-09-23 00:00:05 +00:00), 6),
            TickValue::new(datetime!(2022-09-23 00:00:06 +00:00), 7),
        ] {
            println!("{}", op.next(x)[0]);
        }
    }

    #[test]
    fn push_after() {
        let mut cache = 0;
        let mut op = Periodic::with_circular_n::<2, TickValue<usize>>(Period::secs(2)).build_fn(
            |w, n, x: TickValue<usize>| {
                if n && w.len() > 1 {
                    assert!(w[0] != w[1]);
                }
                if n && w.len() >= 1 {
                    cache = w[0].value;
                } else if w.len() > 1 {
                    assert!(w[1].value == cache);
                }
                x
            },
        );

        for x in [
            TickValue::new(datetime!(2022-09-23 00:00:00 +00:00), 1),
            TickValue::new(datetime!(2022-09-23 00:00:01 +00:00), 2),
            TickValue::new(datetime!(2022-09-23 00:00:02 +00:00), 3),
            TickValue::new(datetime!(2022-09-23 00:00:03 +00:00), 4),
            TickValue::new(datetime!(2022-09-23 00:00:04 +00:00), 5),
            TickValue::new(datetime!(2022-09-23 00:00:05 +00:00), 6),
            TickValue::new(datetime!(2022-09-23 00:00:06 +00:00), 7),
        ] {
            println!("{}", op.next(x)[0]);
        }
    }
}