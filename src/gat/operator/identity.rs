use core::marker::PhantomData;

use super::GatOperator;

/// Identity operator.
pub struct Identity<I>(PhantomData<I>);

impl<I> core::fmt::Debug for Identity<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Identity").finish()
    }
}

impl<I> Clone for Identity<I> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<I> Copy for Identity<I> {}

impl<I> Default for Identity<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I> GatOperator<I> for Identity<I> {
    type Output<'out> = I
    where
        Self: 'out,
        I: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        input
    }
}

/// Create a [`Identity`] operator.
pub fn id<I>() -> Identity<I> {
    Identity(PhantomData)
}
