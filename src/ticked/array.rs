use crate::{Operator, TickValue, Tickable};
use arrayvec::ArrayVec;

/// [`Array`] combinator.
#[derive(Debug, Clone)]
pub struct Array<I, P, const LEN: usize> {
    ops: ArrayVec<P, LEN>,
    _input: core::marker::PhantomData<fn() -> I>,
}

impl<I, T, P, const LEN: usize> Operator<I> for Array<I, P, LEN>
where
    I: Tickable<Value = [T; LEN]>,
    P: Operator<TickValue<T>>,
    P::Output: Tickable,
{
    type Output = TickValue<[<P::Output as Tickable>::Value; LEN]>;

    fn next(&mut self, input: I) -> Self::Output {
        let TickValue { tick, value } = input.into_tick_value();
        let mut idx = 0;
        let res = value.map(|x| {
            let x = TickValue { tick, value: x };
            let y = self.ops[idx].next(x).into_tick_value().value;
            idx += 1;
            y
        });
        TickValue { tick, value: res }
    }
}

/// Apply a ticked operator on an array input to get an array of output.
/// ```
/// use indicator::*;
///
/// fn plus_one<M, I, Q>(mode: M) -> impl Operator<I, Output = TickValue<[usize; 3]>>
/// where
///     M: TumblingWindow,
///     I: Tickable<Value = usize>,
///     Q: QueueCapAtLeast<0, Item = usize>,
/// {
///     cached(mode, |_q: &Q, _n, x: &usize| [*x, *x, *x])
///         .then(array_t(|| map_t(|x| x + 1)))
/// }
/// ```
pub fn array_t<F, I, T, P, const LEN: usize>(mut f: F) -> Array<I, P, LEN>
where
    I: Tickable<Value = [T; LEN]>,
    P: Operator<TickValue<T>>,
    P::Output: Tickable,
    F: FnMut() -> P,
{
    let mut ops = ArrayVec::new();
    for _ in 0..LEN {
        ops.push((f)())
    }
    Array {
        ops,
        _input: core::marker::PhantomData::default(),
    }
}
