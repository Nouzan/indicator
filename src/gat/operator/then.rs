use super::Operator;

/// The [`Operator`] produces by [`then`](super::OperatorExt).
#[derive(Debug, Clone, Copy)]
pub struct Then<P1, P2>(pub(crate) P1, pub(crate) P2);

impl<I, P1, P2> Operator<I> for Then<P1, P2>
where
    P1: Operator<I>,
    P2: for<'out> Operator<P1::Output<'out>>,
{
    type Output<'out> = <P2 as Operator<<P1 as Operator<I>>::Output<'out>>>::Output<'out>
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
