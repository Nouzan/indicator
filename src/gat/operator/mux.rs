use super::GatOperator;

/// Operator returns by [`mux`].
#[derive(Debug, Clone, Copy)]
pub struct Mux<P1, P2>(pub(super) P1, pub(super) P2);

/// Use two operators simultaneously.
/// ```
/// use indicator::gat::*;
///
/// fn plus_mul() -> impl for<'out> GatOperator<usize, Output<'out> = usize> {
///     mux(map(|x| x + 1), map(|x| x * 2)).map(|(x, y)| x + y)
/// }
/// ```
pub fn mux<I, P1, P2>(op1: P1, op2: P2) -> Mux<P1, P2>
where
    I: Clone,
    P1: GatOperator<I>,
    P2: GatOperator<I>,
{
    Mux(op1, op2)
}

impl<I, P1, P2> GatOperator<I> for Mux<P1, P2>
where
    I: Clone,
    P1: GatOperator<I>,
    P2: GatOperator<I>,
{
    type Output<'out> = (P1::Output<'out>, P2::Output<'out>)
    where
        I: 'out,
        P1: 'out,
        P2: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        (self.0.next(input.clone()), self.1.next(input))
    }
}
