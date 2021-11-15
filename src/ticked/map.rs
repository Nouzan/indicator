use crate::{Operator, TickValue, Tickable};

/// [`Map`] operator.
#[derive(Debug, Clone, Copy)]
pub struct Map<P, F> {
    pub(super) source: P,
    pub(super) f: F,
}

impl<I, O, P, F> Operator<I> for Map<P, F>
where
    I: Tickable,
    P: Operator<I>,
    P::Output: Tickable,
    F: FnMut(<P::Output as Tickable>::Value) -> O,
{
    type Output = TickValue<O>;

    fn next(&mut self, input: I) -> Self::Output {
        let TickValue { tick, value } = self.source.next(input).into_tick_value();

        TickValue {
            tick,
            value: (self.f)(value),
        }
    }
}
