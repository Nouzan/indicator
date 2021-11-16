use super::Operator;

/// Identity operator.
#[derive(Debug)]
pub struct Identity<I>(std::marker::PhantomData<fn() -> I>);

/// Create an identity operator.
pub fn id<I>() -> Identity<I> {
    Identity(std::marker::PhantomData::default())
}

impl<I> Clone for Identity<I> {
    fn clone(&self) -> Self {
        id()
    }
}

impl<I> Copy for Identity<I> {}

impl<I> Operator<I> for Identity<I> {
    type Output = I;

    fn next(&mut self, input: I) -> Self::Output {
        input
    }
}
