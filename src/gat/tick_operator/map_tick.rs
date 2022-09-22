use crate::{TickValue, Tickable};

use super::{
    super::operator::map::{map, Map},
    GatOperator,
};

/// Operator returns by [`map_t`].
#[derive(Debug, Clone, Copy)]
pub struct MapTick<P>(pub(super) P);

/// Convert the value of inside the tickabe input directly.
/// ```
/// use indicator::{gat::*, TickValue};
///
/// fn plus_one() -> impl for<'out> GatOperator<TickValue<usize>, Output<'out> = TickValue<usize>> {
///     map_t(|x| x + 1)
/// }
/// ```
pub fn map_t<I, O, F>(f: F) -> MapTick<Map<F>>
where
    F: FnMut(I) -> O,
{
    MapTick(map(f))
}

impl<I, P> GatOperator<I> for MapTick<P>
where
    I: Tickable,
    P: GatOperator<I::Value>,
{
    type Output<'out> = TickValue<P::Output<'out>> where P: 'out, I: 'out;

    #[inline]
    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        input.into_tick_value().map(|v| self.0.next(v))
    }
}
