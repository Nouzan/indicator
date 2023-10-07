use derive_more::{AsRef, From};
use indicator::{prelude::*, IndicatorIteratorExt};
use num::Num;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// An operator for calculating moving average.
// Compared to the implementation in `stack.rs`,
// we removed the concrete types that are not necessary for the implementation,
// with the help of the additional attributes like `#[input]` and `#[prev]`.
#[operator(input = I, generate_out)]
fn ma<T>(#[input] x: &T, #[prev(as_ref)] prev: Option<&T>, #[data(as_ref)] alpha: &T) -> T
where
    T: Send + Sync + 'static,
    T: Num + Clone,
{
    let prev = prev.cloned().unwrap_or_else(|| x.clone());
    x.clone() * alpha.clone() + prev.clone() * (T::one() - alpha.clone())
}

/// Define a stack for calculating moving average.
// We use a const generic here to allow the stack to be reusable.
fn ma_stack<T, P, const N: usize>(alhpa: T) -> BoxLayer<P, T, P::Out>
where
    T: Send + Sync + 'static,
    T: Num + Clone,
    P: ContextOperator<T> + Send + 'static,
{
    id_layer()
        .insert(ma::<T, T, Ma<_, N>, Alpha<_, N>, Ma<_, N>>)
        .provide(Alpha::<_, N>(alhpa))
        .boxed()
}

/// Config for `ma` operator.
#[derive(Debug, Clone, Copy, AsRef)]
struct Alpha<T, const N: usize>(T);

#[derive(Clone, Copy, AsRef, From)]
struct Ma<T, const N: usize>(T);

fn main() -> anyhow::Result<()> {
    let op = output(|_, ctx| ctx.env().get::<Ma<Decimal, 0>>().copied().unwrap())
        .with(ma_stack::<_, _, 0>(dec!(0.3)))
        .inspect(|_, ctx| {
            let v = ctx.env().get::<Ma<Decimal, 1>>().unwrap();
            println!("ma1: {}", v.0);
        })
        .with(ma_stack::<_, _, 1>(dec!(0.5)))
        // We cannot add cache to the stack, because the inner cache will clear the env before the next cache to collect.
        .cache(1)
        .finish();

    let data = [dec!(1), dec!(2), dec!(3)];
    assert_eq!(
        data.into_iter()
            .indicator(op)
            .map(|m| m.0)
            .collect::<Vec<_>>(),
        [dec!(1.0), dec!(1.3), dec!(1.81),]
    );
    Ok(())
}
