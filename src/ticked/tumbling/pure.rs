use core::marker::PhantomData;

use super::{QueueCapAtLeast, Tumbling};

/// Pure tumbling operation.
pub trait PureTumbling<I, Q: QueueCapAtLeast<LEN>, const LEN: usize> {
    /// Output type.
    type Output;

    /// Call pure.
    fn call_pure(q: &Q, y: &mut Option<<Q as QueueCapAtLeast<LEN>>::Item>, x: I) -> Self::Output;

    /// Create a tumbling operation.
    fn into_tumbling() -> PureTumblingOperation<Self> {
        PureTumblingOperation(PhantomData::default())
    }
}

/// Pure tumbling operation.
pub struct PureTumblingOperation<P: ?Sized>(PhantomData<fn() -> P>);

impl<I, Q: QueueCapAtLeast<LEN>, P: PureTumbling<I, Q, LEN>, const LEN: usize> Tumbling<I, Q, LEN>
    for PureTumblingOperation<P>
{
    type Output = P::Output;

    fn call(&mut self, q: &Q, y: &mut Option<Q::Item>, x: I) -> Self::Output {
        P::call_pure(q, y, x)
    }
}
