use crate::{Operator, TickValue, Tickable};

/// [`Tuple`] combinator.
#[derive(Debug, Clone, Copy)]
pub struct Tuple<I, P1, P2>(P1, P2, core::marker::PhantomData<fn() -> I>);

impl<I, I1, I2, P1, P2> Operator<I> for Tuple<I, P1, P2>
where
    I: Tickable<Value = (I1, I2)>,
    P1: Operator<TickValue<I1>>,
    P2: Operator<TickValue<I2>>,
    P1::Output: Tickable,
    P2::Output: Tickable,
{
    type Output = TickValue<(
        <P1::Output as Tickable>::Value,
        <P2::Output as Tickable>::Value,
    )>;

    fn next(&mut self, input: I) -> Self::Output {
        let TickValue {
            tick,
            value: (x1, x2),
        } = input.into_tick_value();
        let x1 = TickValue { tick, value: x1 };
        let x2 = TickValue { tick, value: x2 };
        let y1 = self.0.next(x1).into_tick_value().value;
        let y2 = self.1.next(x2).into_tick_value().value;
        TickValue {
            tick,
            value: (y1, y2),
        }
    }
}

/// Apply ticked operators on tuple input to get output tuple.
/// ```
/// use indicator::*;
///
/// fn plus_one_two<M, I, Q>(mode: M) -> impl Operator<I, Output = TickValue<(usize, usize)>>
/// where
///     M: TumblingWindow,
///     I: Tickable<Value = usize>,
///     Q: QueueCapAtLeast<0, Item = usize>,
/// {
///     cached(mode, |_q: &Q, _n, x: &usize| (*x, *x))
///         .then(tuple_t(map_t(|x| x + 1), map_t(|x| x + 2)))
/// }
/// ```
pub fn tuple_t<I, I1, I2, P1, P2>(p1: P1, p2: P2) -> Tuple<I, P1, P2>
where
    I: Tickable<Value = (I1, I2)>,
    P1: Operator<TickValue<I1>>,
    P2: Operator<TickValue<I2>>,
    P1::Output: Tickable,
    P2::Output: Tickable,
{
    Tuple(p1, p2, core::marker::PhantomData)
}
