use crate::Operator;
use rayon::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

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
