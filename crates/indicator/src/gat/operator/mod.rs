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
pub trait GatOperator<I> {
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

impl<'a, I, P> GatOperator<I> for &'a mut P
where
    P: GatOperator<I>,
{
    type Output<'out> = P::Output<'out>
    where
        Self: 'out,
        I: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        (*self).next(input)
    }
}

/// Helpers for [`GatOperator`].
pub trait GatOperatorExt<I>: GatOperator<I> {
    /// Combine two operators.
    /// ```
    /// use indicator::gat::*;
    ///
    /// fn plus_one() -> impl for<'out> GatOperator<usize, Output<'out> = usize> {
    ///     id().then(map(|x| x + 1))
    /// }
    /// ```
    fn then<P>(self, other: P) -> Then<Self, P>
    where
        Self: Sized,
        P: for<'out> GatOperator<Self::Output<'out>>,
    {
        Then(self, other)
    }

    /// Convert the output.
    /// ```
    /// use indicator::gat::*;
    ///
    /// fn plus_two() -> impl for<'out> GatOperator<usize, Output<'out> = usize> {
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
    /// fn plus_mul() -> impl for<'out> GatOperator<usize, Output<'out> = usize> {
    ///     map(|x| x + 1).mux_with(map(|x| x * 2)).map(|(x, y)| x + y)
    /// }
    /// ```
    fn mux_with<P>(self, other: P) -> Mux<Self, P>
    where
        I: Clone,
        Self: Sized,
        P: GatOperator<I>,
    {
        Mux(self, other)
    }

    /// Convert into a non-GAT operator.
    fn into_operator<O>(self) -> Op<Self>
    where
        Self: for<'out> GatOperator<I, Output<'out> = O> + 'static,
        Self: Sized,
        I: 'static,
    {
        Op(self)
    }
}

impl<I, P> GatOperatorExt<I> for P where P: GatOperator<I> {}

/// Wrapper that Convert `P` into a non-GAT operator.
#[derive(Debug, Clone, Copy)]
pub struct Op<P>(P);

impl<I, O, P> crate::Operator<I> for Op<P>
where
    P: for<'out> GatOperator<I, Output<'out> = O> + 'static,
    I: 'static,
{
    type Output = O;

    #[inline]
    fn next(&mut self, input: I) -> Self::Output {
        self.0.next(input)
    }
}
