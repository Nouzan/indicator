use super::Operator;

/// [`Then`] combinator.
#[derive(Debug, Clone, Copy)]
pub struct Then<I, P1, P2>(
    pub(crate) P1,
    pub(crate) P2,
    pub(crate) std::marker::PhantomData<fn() -> I>,
);

impl<I, P1, P2> Operator<I> for Then<I, P1, P2>
where
    P1: Operator<I>,
    P2: Operator<P1::Output>,
{
    type Output = P2::Output;

    fn next(&mut self, input: I) -> Self::Output {
        self.1.next(self.0.next(input))
    }
}
