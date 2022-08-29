/// Then combinator.
pub mod then;

/// Facet combinator.
pub mod facet;

/// Map operator.
pub mod map;

pub use facet::{facet, Facet};
pub use map::{map, Map};
use then::Then;

#[cfg(feature = "std")]
pub use facet::{facet_map, FacetMap};
extern crate alloc;
use alloc::boxed::Box;

#[cfg(feature = "async")]
use crate::async_operator::Next;

/// Box Operator.
pub type BoxOperator<'a, I, O> = Box<dyn Operator<I, Output = O> + Send + 'a>;

/// Local Box Operator, [`BoxOperator`] without [`Send`].
pub type LocalBoxOperator<'a, I, O> = Box<dyn Operator<I, Output = O> + 'a>;

/// Operator.
pub trait Operator<I> {
    /// Output type.
    type Output;

    /// Produce the next output.
    fn next(&mut self, input: I) -> Self::Output;
}

impl<'a, I, P> Operator<I> for &'a mut P
where
    P: Operator<I> + ?Sized,
{
    type Output = P::Output;

    fn next(&mut self, input: I) -> Self::Output {
        (*self).next(input)
    }
}

/// Operator extension trait.
pub trait OperatorExt<I>: Operator<I> {
    /// Combine with another operator that uses `Self::Output` as input type.
    ///
    /// The result operator will perform the `other` operator after performing the `self`.
    fn then<P2>(self, other: P2) -> Then<I, Self, P2>
    where
        Self: Sized,
        P2: Operator<Self::Output>,
    {
        Then(self, other, core::marker::PhantomData::default())
    }

    /// Combine with another operator with the same input type.
    ///
    /// The result operator will perform two operators simultaneously.
    fn facet<P2>(self, other: P2) -> Facet<I, Self, P2>
    where
        Self: Sized,
        P2: Operator<I>,
    {
        facet(self, other)
    }

    /// Map the output after performing the operator.
    fn map<O, F>(self, f: F) -> Then<I, Self, Map<F>>
    where
        Self: Sized,
        F: FnMut(Self::Output) -> O,
    {
        self.then(map(f))
    }

    /// Convert into a [`BoxOperator`].
    fn boxed<'a>(self) -> BoxOperator<'a, I, Self::Output>
    where
        Self: Sized + Send + 'a,
    {
        Box::new(self)
    }

    /// Convert into a [`LocalBoxOperator`].
    fn boxed_local<'a>(self) -> LocalBoxOperator<'a, I, Self::Output>
    where
        Self: Sized + 'a,
    {
        Box::new(self)
    }

    #[cfg(feature = "async")]
    /// Convert into a [`AsyncOperator`].
    fn into_async_operator(self) -> Next<Self>
    where
        Self: Sized,
    {
        Next { inner: self }
    }
}

impl<I, T: Operator<I>> OperatorExt<I> for T {}

impl<I, P> Operator<I> for Box<P>
where
    P: Operator<I> + ?Sized,
{
    type Output = P::Output;

    fn next(&mut self, input: I) -> Self::Output {
        self.as_mut().next(input)
    }
}
