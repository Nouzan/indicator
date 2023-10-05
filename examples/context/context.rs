use indicator::{prelude::*, IndicatorIteratorExt};
use num::Num;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

struct AddTwo<T>(T);

/// An operator that just add two to the input.
#[operator(input = T)]
fn add_two<T>(In(value): In<&T>) -> AddTwo<T>
where
    T: Num + Clone,
    T: Send + Sync + 'static,
{
    let two = T::one() + T::one();
    AddTwo(value.clone() + two)
}

// Derive `Clone` to satisfy the requirement of `output_with`.
#[derive(Clone)]
struct Ma<T>(T);

/// An operator that does the following:
/// `x => (x + prev(x)) / 2`
#[operator(input = T)]
fn ma<T>(Env(x): Env<&AddTwo<T>>, Prev(prev): Prev<&Ma<T>>) -> Ma<T>
where
    T: Num + Clone,
    T: Send + Sync + 'static,
{
    let two = T::one() + T::one();
    let prev = prev.map(|v| v.0.clone()).unwrap_or(T::zero());
    Ma((x.0.clone() + prev) / two)
}

fn main() -> anyhow::Result<()> {
    let op = output_with(ma)
        .inspect(|value| {
            println!("input: {}", value.value());
            if let Some(AddTwo(x)) = value.context().env().get::<AddTwo<Decimal>>() {
                println!("AddTwo: {x}");
            }
        })
        .insert_env(add_two)
        .cache(1)
        .finish();
    let data = [dec!(1), dec!(2), dec!(3)];

    assert_eq!(
        data.into_iter()
            .indicator(op)
            .map(|Ma(x)| x)
            .collect::<Vec<_>>(),
        [dec!(1.5), dec!(2.75), dec!(3.875)]
    );
    Ok(())
}
