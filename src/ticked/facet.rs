use crate::{Operator, TickValue, Tickable};

#[cfg(feature = "std")]
pub use facet_map::{facet_map_t, FacetMap};

/// [`Facet`] combinator.
#[derive(Debug, Clone, Copy)]
pub struct Facet<I, P1, P2>(
    pub(super) P1,
    pub(super) P2,
    pub(super) std::marker::PhantomData<fn() -> I>,
);

/// Combine two ticked operators into a [`Facet`] ticked operator.
pub fn facet_t<I, P1, P2>(op1: P1, op2: P2) -> Facet<I, P1, P2> {
    Facet(op1, op2, std::marker::PhantomData::default())
}

impl<I: Tickable + Clone, P1, P2> Operator<I> for Facet<I, P1, P2>
where
    P1: Operator<I>,
    P2: Operator<I>,
    P1::Output: Tickable,
    P2::Output: Tickable,
{
    type Output = TickValue<(
        <P1::Output as Tickable>::Value,
        <P2::Output as Tickable>::Value,
    )>;

    fn next(&mut self, input: I) -> Self::Output {
        let tick = *input.tick();
        let o1 = self.0.next(input.clone()).into_tick_value().value;
        let o2 = self.1.next(input).into_tick_value().value;
        TickValue {
            tick,
            value: (o1, o2),
        }
    }
}

#[cfg(feature = "std")]
mod facet_map {
    use crate::{Operator, TickValue, Tickable};
    use std::collections::HashMap;

    /// [`FacetMap`] combinator.
    #[derive(Debug, Clone)]
    pub struct FacetMap<I, P>(HashMap<String, P>, std::marker::PhantomData<fn() -> I>);

    impl<I, P> Operator<I> for FacetMap<I, P>
    where
        I: Tickable + Clone,
        P: Operator<I>,
        P::Output: Tickable,
    {
        type Output = TickValue<HashMap<String, <P::Output as Tickable>::Value>>;

        fn next(&mut self, input: I) -> Self::Output {
            let tick = *input.tick();
            let value = self
                .0
                .iter_mut()
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
    pub fn facet_map_t<I, It, P>(ops: It) -> FacetMap<I, P>
    where
        It: IntoIterator<Item = (String, P)>,
    {
        FacetMap(
            ops.into_iter().collect(),
            std::marker::PhantomData::default(),
        )
    }
}
