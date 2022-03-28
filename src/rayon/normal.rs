use crate::Operator;
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

/// Combine two operators into a [`Facet`] operator.
pub fn facet<I, P1, P2>(op1: P1, op2: P2) -> Facet<I, P1, P2> {
    Facet(op1, op2, core::marker::PhantomData::default())
}

impl<I, P1, P2> Operator<I> for Facet<I, P1, P2>
where
    I: Clone + Send,
    P1: Operator<I> + Send,
    P2: Operator<I> + Send,
    P1::Output: Send,
    P2::Output: Send,
{
    type Output = (P1::Output, P2::Output);

    fn next(&mut self, input: I) -> Self::Output {
        let i1 = input.clone();
        rayon::join(|| self.0.next(i1), || self.1.next(input))
    }
}

/// [`FacetMap`] combinator.
#[derive(Debug, Clone)]
pub struct FacetMap<I, Q, P>(HashMap<Q, P>, core::marker::PhantomData<fn() -> I>);

impl<I, Q, P> Operator<I> for FacetMap<I, Q, P>
where
    I: Clone + Sync,
    Q: Eq + Hash + Clone + Sync + Send,
    P: Operator<I> + Send,
    P::Output: Send,
{
    type Output = HashMap<Q, P::Output>;

    fn next(&mut self, input: I) -> Self::Output {
        self.0
            .par_iter_mut()
            .map(|(k, p)| {
                let o = p.next(input.clone());
                (k.clone(), o)
            })
            .collect()
    }
}

/// Create an operator that apply different operators to the same input,
/// and return the collections of outputs as its output.
///
pub fn facet_map<I, It, Q, P>(ops: It) -> FacetMap<I, Q, P>
where
    It: IntoIterator<Item = (Q, P)>,
    I: Clone + Sync,
    Q: Eq + Hash + Clone + Sync + Send,
    P: Operator<I> + Send,
    P::Output: Send,
{
    FacetMap(
        ops.into_iter().collect(),
        core::marker::PhantomData::default(),
    )
}
