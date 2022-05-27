use super::Operator;

#[cfg(feature = "std")]
pub use facet_map::{facet_map, FacetMap};

/// [`Facet`] combinator.
#[derive(Debug, Clone, Copy)]
pub struct Facet<I, P1, P2>(
    pub(crate) P1,
    pub(crate) P2,
    pub(crate) core::marker::PhantomData<fn() -> I>,
);

/// Combine two operators into a [`Facet`] operator.
pub fn facet<I, P1, P2>(op1: P1, op2: P2) -> Facet<I, P1, P2> {
    Facet(op1, op2, core::marker::PhantomData::default())
}

impl<I: Clone, P1, P2> Operator<I> for Facet<I, P1, P2>
where
    P1: Operator<I>,
    P2: Operator<I>,
{
    type Output = (P1::Output, P2::Output);

    fn next(&mut self, input: I) -> Self::Output {
        (self.0.next(input.clone()), self.1.next(input))
    }
}

#[cfg(feature = "std")]
mod facet_map {
    use super::Operator;
    use std::collections::HashMap;
    use std::hash::Hash;

    /// [`FacetMap`] combinator.
    #[derive(Debug, Clone)]
    pub struct FacetMap<I, Q, P>(
        pub(crate) HashMap<Q, P>,
        core::marker::PhantomData<fn() -> I>,
    );

    impl<I, Q, P> Operator<I> for FacetMap<I, Q, P>
    where
        I: Clone,
        Q: Eq + Hash + Clone,
        P: Operator<I>,
    {
        type Output = HashMap<Q, P::Output>;

        fn next(&mut self, input: I) -> Self::Output {
            self.0
                .iter_mut()
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
        Q: Eq + Hash + Clone,
        It: IntoIterator<Item = (Q, P)>,
    {
        FacetMap(
            ops.into_iter().collect(),
            core::marker::PhantomData::default(),
        )
    }
}
