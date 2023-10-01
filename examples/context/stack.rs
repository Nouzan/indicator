use indicator::{
    context::{AddData, Cache, Insert},
    prelude::*,
    IndicatorIteratorExt,
};
use num::Num;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Config for `ma` operator.
#[derive(Debug, Clone, Copy)]
struct Alpha<T>(T);

#[operator(T)]
fn ma<T>(In(x): In<&T>, Prev(prev): Prev<&T>, Data(alpha): Data<&Alpha<T>>) -> T
where
    T: Send + Sync + 'static,
    T: Num + Clone,
{
    let prev = prev.cloned().unwrap_or_else(|| x.clone());
    x.clone() * alpha.0.clone() + prev.clone() * (T::one() - alpha.0.clone())
}

fn ma_stack<T, P>(alhpa: T) -> impl Layer<T, P, Out = P::Out>
where
    T: Send + Sync + 'static,
    T: Num + Clone,
    P: ContextOperator<T>,
{
    id_layer()
        .with(Insert(ma))
        .with(AddData::with_data(Alpha(alhpa)))
}

fn main() -> anyhow::Result<()> {
    let op = output(|_, ctx| ctx.env().get::<Decimal>().copied().unwrap())
        .with(ma_stack(dec!(0.3)))
        .with(Cache::new())
        .finish();

    let data = [dec!(1), dec!(2), dec!(3)];
    assert_eq!(
        data.into_iter().indicator(op).collect::<Vec<_>>(),
        [dec!(1.0), dec!(1.3), dec!(1.81),]
    );
    Ok(())
}
