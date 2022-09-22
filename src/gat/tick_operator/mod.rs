use crate::Tickable;

use self::map_tick::MapTick;

use super::{operator::then::Then, GatOperator};

/// Map over the ticked value.
pub mod map_tick;

/// Helpers for the operaors with [`Tickable`] output.
pub trait TickGatOperatorExt<I>: GatOperator<I>
where
    for<'out> Self::Output<'out>: Tickable,
{
    /// Map over the output tick value.
    /// ```
    /// use indicator::{gat::*, TickValue};
    ///
    /// fn plus_one() -> impl for<'out> GatOperator<TickValue<usize>, Output<'out> = TickValue<usize>> {
    ///     id().map_tick(map(|x| x + 1))
    /// }
    /// ```
    fn map_tick<P>(self, op: P) -> Then<Self, MapTick<P>>
    where
        Self: Sized,
        P: for<'out> GatOperator<<Self::Output<'out> as Tickable>::Value>,
    {
        Then(self, MapTick(op))
    }
}

impl<I, P> TickGatOperatorExt<I> for P
where
    P: GatOperator<I>,
    for<'out> P::Output<'out>: Tickable,
{
}
