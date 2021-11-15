use super::Operator;

/// [`Map`] operator.
#[derive(Debug, Clone, Copy)]
pub struct Map<F> {
    pub(super) f: F,
}

/// Create a [`Map`] operator.
pub fn map<I, O, F>(f: F) -> Map<F>
where
    F: FnMut(I) -> O,
{
    Map { f }
}

impl<I, O, F> Operator<I> for Map<F>
where
    F: FnMut(I) -> O,
{
    type Output = O;

    fn next(&mut self, input: I) -> Self::Output {
        (self.f)(input)
    }
}
