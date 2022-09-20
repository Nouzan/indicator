use crate::Tickable;

use self::tick_map::TickMap;

use super::{map_t, operator::then::Then, Operator};

/// Map over the ticked value.
pub mod tick_map;

/// Helpers for the operaors with [`Tickable`] output.
pub trait TickOperatorExt<I>: Operator<I>
where
    for<'out> Self::Output<'out>: Tickable,
{
    /// Map over the output tick value.
    /// ```
    /// use indicator::{gat::*, TickValue};
    ///
    /// fn plus_one() -> impl for<'out> Operator<TickValue<usize>, Output<'out> = TickValue<usize>> {
    ///     id().map_t(|x| x + 1)
    /// }
    /// ```
    fn map_t<U, F>(self, f: F) -> Then<Self, TickMap<F>>
    where
        Self: Sized,
        F: FnMut(<Self::Output<'_> as Tickable>::Value) -> U,
    {
        Then(self, map_t(f))
    }
}

impl<I, P> TickOperatorExt<I> for P
where
    P: Operator<I>,
    for<'out> P::Output<'out>: Tickable,
{
}
