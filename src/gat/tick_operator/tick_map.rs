use crate::{TickValue, Tickable};

use super::GatOperator;

/// Operator returns by [`map_t`].
#[derive(Debug, Clone, Copy)]
pub struct TickMap<F>(pub(super) F);

/// Convert the value of inside the tickabe input directly.
/// ```
/// use indicator::{gat::*, TickValue};
///
/// fn plus_one() -> impl for<'out> GatOperator<TickValue<usize>, Output<'out> = TickValue<usize>> {
///     map_t(|x| x + 1)
/// }
/// ```
pub fn map_t<I, O, F>(f: F) -> TickMap<F>
where
    F: FnMut(I) -> O,
{
    TickMap(f)
}

impl<I, O, F> GatOperator<I> for TickMap<F>
where
    I: Tickable,
    F: FnMut(I::Value) -> O,
{
    type Output<'out> = TickValue<O> where F: 'out, I: 'out;

    #[inline]
    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        input.into_tick_value().map(&mut self.0)
    }
}
