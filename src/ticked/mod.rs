/// Tumbling-windowed operator.
pub mod tumbling;

/// [`Facet`] combinator of ticked operators.
pub mod facet;

/// [`Map`] combinator of ticked operators.
pub mod map;

#[cfg(feature = "array-vec")]
/// [`Array`] combinator of ticked operators.
pub mod array;

/// [`Tuple`] combinator of ticked operators.
pub mod tuple;

use crate::operator::then::Then;
use crate::{Operator, OperatorExt, Tickable};
#[cfg(feature = "array-vec")]
pub use array::{array_t, Array};
#[cfg(feature = "std")]
pub use facet::{facet_map_t, FacetMap};
pub use facet::{facet_t, Facet};
pub use map::{map_t, Map};
pub use tumbling::{
    cached, queue::QueueCapAtLeast, Cached, CachedOperation, TumblingOperation, TumblingOperator,
    TumblingQueue,
};
#[cfg(feature = "std")]
pub use tumbling::{shared, SharedMap};
pub use tuple::{tuple_t, Tuple};

/// Ticked operator.
pub trait TickedOperatorExt<I: Tickable>: Operator<I>
where
    <Self as Operator<I>>::Output: Tickable,
{
    /// Combine with the other tick operator to get a facet operator that keep the [`Tick`] unchanged.
    ///
    /// [`Tick`]: crate::Tick
    fn facet_t<P2: Operator<I>>(self, other: P2) -> Facet<I, Self, P2>
    where
        Self: Sized,
        <P2 as Operator<I>>::Output: Tickable,
    {
        facet_t(self, other)
    }

    /// Transform the value of the output but keep the [`Tick`] unchanged.
    ///
    /// [`Tick`]: crate::Tick
    fn map_t<O, F>(self, f: F) -> Then<I, Self, Map<F>>
    where
        Self: Sized,
        F: FnMut(<Self::Output as Tickable>::Value) -> O,
    {
        self.then(map_t(f))
    }
}

impl<P, I: Tickable> TickedOperatorExt<I> for P
where
    P: Operator<I>,
    <P as Operator<I>>::Output: Tickable,
{
}
