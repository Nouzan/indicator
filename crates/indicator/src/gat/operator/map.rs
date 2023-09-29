use super::GatOperator;

/// Operator returns by [`map`].
#[derive(Debug, Clone, Copy)]
pub struct Map<F>(pub(super) F);

/// Convert the input directly.
/// ```
/// use indicator::gat::*;
///
/// fn plus_one() -> impl for<'out> GatOperator<usize, Output<'out> = usize> {
///     map(|x| x + 1)
/// }
/// ```
pub fn map<I, O, F>(f: F) -> Map<F>
where
    F: FnMut(I) -> O,
{
    Map(f)
}

impl<I, O, F> GatOperator<I> for Map<F>
where
    F: FnMut(I) -> O,
{
    type Output<'out> = O where F: 'out, I: 'out;

    #[inline]
    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        (self.0)(input)
    }
}
