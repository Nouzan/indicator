use super::Operator;

/// [`Map`] operator.
#[derive(Debug, Clone, Copy)]
pub struct Map<P, F> {
    pub(super) source: P,
    pub(super) f: F,
}

impl<I, O, P, F> Operator<I> for Map<P, F>
where
    P: Operator<I>,
    F: FnMut(P::Output) -> O,
{
    type Output = O;

    fn next(&mut self, input: I) -> Self::Output {
        (self.f)(self.source.next(input))
    }
}
