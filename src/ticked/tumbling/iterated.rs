use crate::{tumbling, QueueCapAtLeast, TumblingOperation, TumblingOperator, TumblingWindow};

/// Tumbling operations that caches outputs.
pub trait IteratedOperation<I, Q: QueueCapAtLeast<LEN, Item = Self::Output>, const LEN: usize> {
    /// The output type.
    type Output: Clone;

    /// Call.
    fn call(&mut self, q: &Q, y: Option<&Self::Output>, x: I) -> Self::Output;
}

impl<F, I, O, Q, const LEN: usize> IteratedOperation<I, Q, LEN> for F
where
    O: Clone,
    Q: QueueCapAtLeast<LEN, Item = O>,
    F: FnMut(&Q, Option<&O>, I) -> O,
{
    type Output = O;

    fn call(&mut self, q: &Q, y: Option<&Self::Output>, x: I) -> Self::Output {
        (self)(q, y, x)
    }
}

/// A tumbling operation that caches outputs.
#[derive(Debug, Clone, Copy)]
pub struct Iterated<P>(P);

impl<I, Q, P, const LEN: usize> TumblingOperation<I, Q, LEN> for Iterated<P>
where
    Q: QueueCapAtLeast<LEN, Item = P::Output>,
    P: IteratedOperation<I, Q, LEN>,
{
    type Output = P::Output;

    fn call(&mut self, q: &Q, y: &mut Option<Q::Item>, x: I) -> Self::Output {
        let output = self.0.call(q, y.as_ref(), x);
        *y = Some(output.clone());
        output
    }
}

/// Create a iterated tumbling operator from the given operation.
/// ```
/// use indicator::*;
/// use rust_decimal::Decimal;
///
/// fn ohlc<M, I, Q>(mode: M) -> impl Operator<I, Output = TickValue<[Decimal; 4]>>
/// where
///     M: TumblingWindow,
///     I: Tickable<Value = Decimal>,
///     Q: QueueCapAtLeast<0, Item = [Decimal; 4]>,
/// {
///     iterated(mode, |_q: &Q, y: Option<&[Decimal; 4]>, x| {
///         match y {
///             Some(y) => [y[0], y[1].max(x), y[2].min(x), x],
///             None => [x, x, x, x]
///         }
///     })
/// }
/// ```
pub fn iterated<M, I, Q, P, const LEN: usize>(
    mode: M,
    op: P,
) -> TumblingOperator<M, Q, Iterated<P>, LEN>
where
    M: TumblingWindow,
    Q: QueueCapAtLeast<LEN, Item = P::Output>,
    P: IteratedOperation<I, Q, LEN>,
{
    tumbling(mode, Iterated(op))
}
