use super::GatOperator;

/// The [`GatOperator`] produces by [`then`](super::OperatorExt).
#[derive(Debug, Clone, Copy)]
pub struct Then<P1, P2>(pub(crate) P1, pub(crate) P2);

impl<I, P1, P2> GatOperator<I> for Then<P1, P2>
where
    P1: GatOperator<I>,
    P2: for<'out> GatOperator<P1::Output<'out>>,
{
    type Output<'out> = <P2 as GatOperator<<P1 as GatOperator<I>>::Output<'out>>>::Output<'out>
    where
        I: 'out,
        P1: 'out,
        P2: 'out;

    #[inline]
    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        self.1.next(self.0.next(input))
    }
}
