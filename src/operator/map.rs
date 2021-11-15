use super::Operator;

/// [`Map`] operator.
#[derive(Debug, Clone, Copy)]
pub struct Map<I, P, F> {
    pub(super) source: P,
    pub(super) f: F,
    pub(super) _input: std::marker::PhantomData<fn() -> I>,
}

impl<I, O, P, F> Operator<I> for Map<I, P, F>
where
    P: Operator<I>,
    F: FnMut(P::Output) -> O,
{
    type Output = O;

    fn next(&mut self, input: I) -> Self::Output {
        (self.f)(self.source.next(input))
    }
}
