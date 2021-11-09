/// Queue.
pub mod queue;

/// Pure tumbling operation.
pub mod pure;

use crate::{Operator, TickValue, Tickable, TumblingWindow};
pub use queue::{QueueCapAtLeast, TumblingQueue};

/// Tumbling operation.
pub trait Tumbling<I, Q: QueueCapAtLeast<LEN>, const LEN: usize> {
    /// Output type.
    type Output;

    /// Call.
    fn call(&mut self, q: &Q, y: &mut Option<Q::Item>, x: I) -> Self::Output;

    /// Convert into an operator.
    fn into_operator<M: TumblingWindow>(self, mode: M) -> TumblingOperator<M, Q, Self, LEN>
    where
        Self: Sized,
    {
        TumblingOperator {
            queue: TumblingQueue::new(mode),
            acc: None,
            op: self,
        }
    }
}

impl<F, I, O, Q: QueueCapAtLeast<LEN>, const LEN: usize> Tumbling<I, Q, LEN> for F
where
    F: FnMut(&Q, &mut Option<Q::Item>, I) -> O,
{
    type Output = O;

    fn call(&mut self, q: &Q, y: &mut Option<Q::Item>, x: I) -> O {
        (self)(q, y, x)
    }
}

/// Tumbling operator.
pub struct TumblingOperator<M: TumblingWindow, Q: QueueCapAtLeast<LEN>, P, const LEN: usize> {
    queue: TumblingQueue<M, Q, LEN>,
    acc: Option<Q::Item>,
    op: P,
}

impl<M: TumblingWindow + Clone, Q: QueueCapAtLeast<LEN>, P: Clone, const LEN: usize> Clone
    for TumblingOperator<M, Q, P, LEN>
{
    fn clone(&self) -> Self {
        Self {
            queue: TumblingQueue::new(self.queue.mode.clone()),
            acc: None,
            op: self.op.clone(),
        }
    }
}

impl<
        M: TumblingWindow,
        I: Tickable,
        Q: QueueCapAtLeast<LEN>,
        P: Tumbling<I::Value, Q, LEN>,
        const LEN: usize,
    > Operator<I> for TumblingOperator<M, Q, P, LEN>
{
    type Output = TickValue<P::Output>;

    fn next(&mut self, input: I) -> Self::Output {
        let TickValue { tick, value } = input.into_tick_value();
        self.queue.enque_or_ignore(&tick, &mut self.acc);
        let res = self.op.call(&self.queue.queue, &mut self.acc, value);
        TickValue { tick, value: res }
    }
}
