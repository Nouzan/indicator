/// Then combinator.
pub mod then;

/// Facet combinator.
pub mod facet;

/// Map operator.
pub mod map;

use facet::Facet;
use map::Map;
use then::Then;

#[cfg(feature = "std")]
pub use facet::{facet_map, FacetMap};

/// Operator.
pub trait Operator<I> {
    /// Output type.
    type Output;

    /// Produce the next output.
    fn next(&mut self, input: I) -> Self::Output;
}

/// Operator extension trait.
pub trait OperatorExt<I>: Operator<I> {
    /// Combine with another operator that uses `Self::Output` as input type.
    ///
    /// The result operator will perform the `other` operator after performing the `self`.
    fn then<P2>(self, other: P2) -> Then<Self, P2>
    where
        Self: Sized,
        P2: Operator<Self::Output>,
    {
        Then(self, other)
    }

    /// Combine with another operator with the same input type.
    ///
    /// The result operator will perform two operators simultaneously.
    fn facet<P2>(self, other: P2) -> Facet<Self, P2>
    where
        Self: Sized,
        P2: Operator<I>,
    {
        Facet(self, other)
    }

    /// Map the output after performing the operator.
    fn map<O, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> O,
    {
        Map { source: self, f }
    }
}

impl<I, T: Operator<I>> OperatorExt<I> for T {}

/// Into operator.
pub trait IntoOperator<I> {
    /// The result operator.
    type Operator: Operator<I>;

    /// Convert into operator.
    fn into_operator(self) -> Self::Operator;
}
