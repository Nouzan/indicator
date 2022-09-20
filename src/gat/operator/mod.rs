use self::{map::Map, mux::Mux, then::Then};

/// Combine two operators.
pub mod then;

/// Convert the input directly.
pub mod map;

/// Use two operators simultaneously.
pub mod mux;

/// Identity operator.
pub mod identity;

/// Operator that produces indicator values by calling `next` method.
/// It is just a version of `FnMut` with generic associated (lifetime) output.
pub trait Operator<I> {
    /// The output type.
    type Output<'out>
    where
        Self: 'out,
        I: 'out;

    /// Produce the next indicator value according to the given input.
    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out;
}

/// Helpers for [`Operator`].
pub trait OperatorExt<I>: Operator<I> {
    /// Combine two operators.
    /// ```
    /// use indicator::gat::*;
    ///
    /// fn plus_one() -> impl for<'out> Operator<usize, Output<'out> = usize> {
    ///     id().then(map(|x| x + 1))
    /// }
    /// ```
    fn then<P>(self, other: P) -> Then<Self, P>
    where
        Self: Sized,
        P: for<'out> Operator<Self::Output<'out>>,
    {
        Then(self, other)
    }

    /// Convert the output.
    /// ```
    /// use indicator::gat::*;
    ///
    /// fn plus_two() -> impl for<'out> Operator<usize, Output<'out> = usize> {
    ///     id().then(map(|x| x + 2))
    /// }
    /// ```
    fn map<O, F>(self, f: F) -> Then<Self, Map<F>>
    where
        Self: Sized,
        F: FnMut(Self::Output<'_>) -> O,
    {
        Then(self, Map(f))
    }

    /// Use with the other operator simultaneously.
    /// ```
    /// use indicator::gat::*;
    ///
    /// fn plus_mul() -> impl for<'out> Operator<usize, Output<'out> = usize> {
    ///     map(|x| x + 1).mux_with(map(|x| x * 2)).map(|(x, y)| x + y)
    /// }
    /// ```
    fn mux_with<P>(self, other: P) -> Mux<Self, P>
    where
        I: Clone,
        Self: Sized,
        P: Operator<I>,
    {
        Mux(self, other)
    }
}

impl<I, P> OperatorExt<I> for P where P: Operator<I> {}
