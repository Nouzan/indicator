use super::Operator;

#[cfg(feature = "std")]
pub use facet_map::{facet_map, FacetMap};

/// [`Facet`] combinator.
#[derive(Debug, Clone, Copy)]
pub struct Facet<P1, P2>(pub(super) P1, pub(super) P2);

impl<I: Clone, P1, P2> Operator<I> for Facet<P1, P2>
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

    /// [`FacetMap`] combinator.
    #[derive(Debug, Clone)]
    pub struct FacetMap<P>(HashMap<String, P>);

    impl<I, P> Operator<I> for FacetMap<P>
    where
        I: Clone,
        P: Operator<I>,
    {
        type Output = HashMap<String, P::Output>;

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
    pub fn facet_map<It, P>(ops: It) -> FacetMap<P>
    where
        It: IntoIterator<Item = (String, P)>,
    {
        FacetMap(ops.into_iter().collect())
    }
}
