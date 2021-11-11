/// Tumbling-windowed operator.
pub mod tumbling;

/// [`Facet`] combinator of ticked operators.
pub mod facet;

/// [`Map`] operator of ticked operators.
pub mod map;

use crate::{Operator, Tickable};
pub use facet::Facet;
#[cfg(feature = "std")]
pub use facet::{facet_map_t, FacetMap};
pub use map::Map;
pub use tumbling::{queue::QueueCapAtLeast, TumblingOperation, TumblingOperator, TumblingQueue};

/// Ticked operator.
pub trait TickedOperatorExt<I: Tickable>: Operator<I>
where
    <Self as Operator<I>>::Output: Tickable,
{
    /// Combine with the other tick operator to get a facet operator that keep the [`Tick`] unchanged.
    ///
    /// [`Tick`]: crate::Tick
    fn facet_t<P2: Operator<I>>(self, other: P2) -> Facet<Self, P2>
    where
        Self: Sized,
        <P2 as Operator<I>>::Output: Tickable,
    {
        Facet(self, other)
    }

    /// Transform the value of the output but keep the [`Tick`] unchanged.
    ///
    /// [`Tick`]: crate::Tick
    fn map_t<O, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(<Self::Output as Tickable>::Value) -> O,
    {
        Map { source: self, f }
    }
}

impl<P, I: Tickable> TickedOperatorExt<I> for P
where
    P: Operator<I>,
    <P as Operator<I>>::Output: Tickable,
{
}
