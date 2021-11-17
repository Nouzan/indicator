use crate::{tumbling, QueueCapAtLeast, TumblingOperation, TumblingOperator, TumblingWindow};

/// Tumbling operations that apply on the cached inputs.
pub trait CachedOperation<I, Q: QueueCapAtLeast<LEN, Item = I>, const LEN: usize> {
    /// The output type.
    type Output;

    /// Call.
    fn call(&mut self, q: &Q, new_period: bool, x: &I) -> Self::Output;

    /// Share the cached queue with other cached operation.
    fn share_with<P>(self, op: P) -> Shared<I, Self, P>
    where
        P: CachedOperation<I, Q, LEN>,
        Self: Sized,
    {
        Shared(self, op, core::marker::PhantomData::default())
    }
}

impl<F, I, O, Q: QueueCapAtLeast<LEN, Item = I>, const LEN: usize> CachedOperation<I, Q, LEN> for F
where
    F: FnMut(&Q, bool, &I) -> O,
{
    type Output = O;

    fn call(&mut self, q: &Q, new_period: bool, x: &I) -> O {
        (self)(q, new_period, x)
    }
}

/// A combinated cached operation of two cached operations sharing the same queue.
#[derive(Debug, Clone, Copy)]
pub struct Shared<I, P1, P2>(P1, P2, core::marker::PhantomData<fn() -> I>);

impl<I, Q: QueueCapAtLeast<LEN, Item = I>, P1, P2, const LEN: usize> CachedOperation<I, Q, LEN>
    for Shared<I, P1, P2>
where
    P1: CachedOperation<I, Q, LEN>,
    P2: CachedOperation<I, Q, LEN>,
{
    type Output = (P1::Output, P2::Output);

    fn call(&mut self, q: &Q, new_period: bool, x: &I) -> Self::Output {
        (self.0.call(q, new_period, x), self.1.call(q, new_period, x))
    }
}

/// A tumbling operation that only apply on the input-cached queue.
#[derive(Debug, Clone, Copy)]
pub struct Cached<P>(P);

impl<I, Q: QueueCapAtLeast<LEN, Item = I>, P, const LEN: usize> TumblingOperation<I, Q, LEN>
    for Cached<P>
where
    P: CachedOperation<I, Q, LEN>,
{
    type Output = P::Output;

    fn call(&mut self, q: &Q, y: &mut Option<Q::Item>, x: I) -> Self::Output {
        let new_period = y.is_none();
        *y = Some(x);
        self.0.call(q, new_period, y.as_ref().unwrap())
    }
}

/// Create a cached tumbling operator.
pub fn cached<M: TumblingWindow, I, Q: QueueCapAtLeast<LEN, Item = I>, P, const LEN: usize>(
    mode: M,
    op: P,
) -> TumblingOperator<M, Q, Cached<P>, LEN>
where
    P: CachedOperation<I, Q, LEN>,
{
    tumbling(mode, Cached(op))
}

#[cfg(feature = "std")]
/// Map version of shared queue cached operations.
pub mod shared_map {
    use super::{CachedOperation, QueueCapAtLeast};
    use std::marker::PhantomData;
    use std::{collections::HashMap, hash::Hash};

    /// A map of cached operations sharing the same queue.
    #[derive(Debug, Clone)]
    pub struct SharedMap<I, K, P>(HashMap<K, P>, PhantomData<fn() -> I>);

    impl<I, Q: QueueCapAtLeast<LEN, Item = I>, K, P, const LEN: usize> CachedOperation<I, Q, LEN>
        for SharedMap<I, K, P>
    where
        K: Clone + Eq + Hash,
        P: CachedOperation<I, Q, LEN>,
    {
        type Output = HashMap<K, P::Output>;

        fn call(&mut self, q: &Q, new_period: bool, x: &I) -> Self::Output {
            self.0
                .iter_mut()
                .map(|(k, p)| (k.clone(), p.call(q, new_period, x)))
                .collect()
        }
    }

    /// Create a map of cached operations sharing the same input queue.
    pub fn shared<I, Q: QueueCapAtLeast<LEN, Item = I>, K, P, const LEN: usize>(
        map: HashMap<K, P>,
    ) -> SharedMap<I, K, P>
    where
        K: Clone + Eq + Hash,
        P: CachedOperation<I, Q, LEN>,
    {
        SharedMap(map, PhantomData::default())
    }
}
