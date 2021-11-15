use crate::{Operator, TickValue, Tickable};

/// [`Map`] operator for [`map_t`].
#[derive(Debug, Clone, Copy)]
pub struct Map<F> {
    pub(super) f: F,
}

/// Create a [`Map`] ticked operator.
pub fn map_t<F>(f: F) -> Map<F> {
    Map { f }
}

impl<I, O, F> Operator<I> for Map<F>
where
    I: Tickable,
    F: FnMut(<I as Tickable>::Value) -> O,
{
    type Output = TickValue<O>;

    fn next(&mut self, input: I) -> Self::Output {
        let TickValue { tick, value } = input.into_tick_value();

        TickValue {
            tick,
            value: (self.f)(value),
        }
    }
}
