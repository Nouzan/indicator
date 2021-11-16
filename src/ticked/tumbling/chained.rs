use super::{QueueCapAtLeast, TumblingOperation, TumblingOperator, TumblingWindow};
use crate::{
    operator::{
        id,
        then::{then, Then},
        Identity, Operator,
    },
    ticked::{facet_t, Facet},
    Tickable,
};

/// Tumbling operators.
#[derive(Debug, Clone, Copy)]
pub struct Chained<M, I, P1, P2> {
    mode: M,
    op: Then<I, P1, P2>,
}

/// Build a tumbling operator.
pub fn tumbling<M: TumblingWindow, I>(mode: M) -> Chained<M, I, Identity<I>, Identity<I>> {
    Chained {
        mode,
        op: then(id(), id()),
    }
}

impl<M, I, P1, P2> Chained<M, I, P1, P2>
where
    P1: Operator<I>,
    P2: Operator<P1::Output>,
{
    /// Chain up with next tumbling operation.
    pub fn w<Q, P, const LEN: usize>(
        self,
        op: P,
    ) -> Chained<M, I, Then<I, P1, P2>, TumblingOperator<M, Q, P, LEN>>
    where
        M: TumblingWindow,
        Q: QueueCapAtLeast<LEN>,
        P2::Output: Tickable,
        P: TumblingOperation<<P2::Output as Tickable>::Value, Q, LEN>,
    {
        let op = then(self.op, TumblingOperator::new(self.mode.clone(), op));

        Chained {
            mode: self.mode,
            op,
        }
    }

    /// Multiplexing the input with other [`TumblingOperation`].
    pub fn mux_with<Q, P, const LEN: usize>(
        self,
        op: P,
    ) -> Chained<M, I, P1, Facet<I, P2, TumblingOperator<M, Q, P, LEN>>>
    where
        M: TumblingWindow,
        Q: QueueCapAtLeast<LEN>,
        P1::Output: Tickable,
        P: TumblingOperation<<P1::Output as Tickable>::Value, Q, LEN>,
    {
        let op = TumblingOperator::new(self.mode.clone(), op);
        let op = then(self.op.0, facet_t(self.op.1, op));
        Chained {
            mode: self.mode,
            op,
        }
    }
}

impl<M, I, P1, P2> Operator<I> for Chained<M, I, P1, P2>
where
    P1: Operator<I>,
    P2: Operator<P1::Output>,
{
    type Output = <Then<I, P1, P2> as Operator<I>>::Output;

    fn next(&mut self, input: I) -> Self::Output {
        self.op.next(input)
    }
}
