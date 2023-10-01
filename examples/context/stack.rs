use indicator::{prelude::*, IndicatorIteratorExt};
use num::Num;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Define a stack for calculating moving average.
// We use a const generic here to allow the stack to be reusable.
fn ma_stack<T, P, const N: usize>(alhpa: T) -> impl Layer<T, P, Out = P::Out>
where
    T: Send + Sync + 'static,
    T: Num + Clone,
    P: ContextOperator<T>,
{
    id_layer().insert(ma::<_, N>).provide(Alpha::<_, N>(alhpa))
}

/// Config for `ma` operator.
#[derive(Debug, Clone, Copy)]
struct Alpha<T, const N: usize>(T);

#[derive(Clone, Copy)]
struct Ma<T, const N: usize>(T);

#[operator(T)]
fn ma<T, const N: usize>(
    In(x): In<&T>,
    Prev(prev): Prev<&Ma<T, N>>,
    Data(alpha): Data<&Alpha<T, N>>,
) -> Ma<T, N>
where
    T: Send + Sync + 'static,
    T: Num + Clone,
{
    let prev = prev.cloned().map(|x| x.0).unwrap_or_else(|| x.clone());
    Ma(x.clone() * alpha.0.clone() + prev.clone() * (T::one() - alpha.0.clone()))
}

fn main() -> anyhow::Result<()> {
    let op = output(|_, ctx| ctx.env().get::<Ma<Decimal, 0>>().copied().unwrap())
        .with(ma_stack::<_, _, 0>(dec!(0.3)))
        .inspect(|v| {
            let v = v.context.env().get::<Ma<Decimal, 1>>().unwrap();
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
