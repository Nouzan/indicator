use crate::{Operator, TickValue, Tickable};
use rayon::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

/// [`Facet`] combinator.
#[derive(Debug, Clone, Copy)]
pub struct Facet<I, P1, P2>(
    pub(super) P1,
    pub(super) P2,
    pub(super) core::marker::PhantomData<fn() -> I>,
);

/// Combine two ticked operators into a [`Facet`] ticked operator.
pub fn facet_t<I, P1, P2>(op1: P1, op2: P2) -> Facet<I, P1, P2> {
    Facet(op1, op2, core::marker::PhantomData::default())
}

impl<I, P1, P2> Operator<I> for Facet<I, P1, P2>
where
    I: Tickable + Clone + Send,
    P1: Operator<I> + Send,
    P2: Operator<I> + Send,
    P1::Output: Tickable + Send,
    P2::Output: Tickable + Send,
    <P1::Output as Tickable>::Value: Send,
    <P2::Output as Tickable>::Value: Send,
{
    type Output = TickValue<(
        <P1::Output as Tickable>::Value,
        <P2::Output as Tickable>::Value,
    )>;

    fn next(&mut self, input: I) -> Self::Output {
        let tick = input.tick();
        let i1 = input.clone();
        let (o1, o2) = rayon::join(
            || self.0.next(i1).into_tick_value().value,
            || self.1.next(input).into_tick_value().value,
        );
        TickValue {
            tick,
            value: (o1, o2),
        }
    }
}

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
        let tick = input.tick();
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
