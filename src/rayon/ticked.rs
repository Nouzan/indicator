use crate::{Operator, TickValue, Tickable};
use rayon::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

/// [`FacetMap`] combinator.
#[derive(Debug, Clone)]
pub struct FacetMap<I, Q, P>(HashMap<Q, P>, core::marker::PhantomData<fn() -> I>);

impl<I, Q, P> Operator<I> for FacetMap<I, Q, P>
where
    I: Tickable + Clone + Sync,
    Q: Eq + Hash + Clone + Sync + Send,
    P: Operator<I> + Send,
    P::Output: Tickable,
    <P::Output as Tickable>::Value: Send,
{
    type Output = TickValue<HashMap<Q, <P::Output as Tickable>::Value>>;

    fn next(&mut self, input: I) -> Self::Output {
        let tick = *input.tick();
        let value = self
            .0
            .par_iter_mut()
            .map(|(k, p)| {
                let o = p.next(input.clone()).into_tick_value().value;
                (k.clone(), o)
            })
            .collect();
        TickValue { tick, value }
    }
}

/// Create an operator that apply different operators to the same input,
/// and return the collections of outputs as its output.
pub fn facet_map_t<I, It, Q, P>(ops: It) -> FacetMap<I, Q, P>
where
    It: IntoIterator<Item = (Q, P)>,
    I: Tickable + Clone + Sync,
    Q: Eq + Hash + Clone + Sync + Send,
    P: Operator<I> + Send,
    P::Output: Tickable,
    <P::Output as Tickable>::Value: Send,
{
    FacetMap(
        ops.into_iter().collect(),
        core::marker::PhantomData::default(),
    )
}
