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
fn ma<T>(In(input): In<&T>, Env(x): Env<Option<&AddTwo<T>>>, Prev(prev): Prev<&Ma<T>>) -> Ma<T>
where
    T: Num + Clone,
    T: Send + Sync + 'static,
{
    let two = T::one() + T::one();
    let prev = prev.map(|v| v.0.clone()).unwrap_or(T::zero());
    let x = x.map(|v| v.0.clone()).unwrap_or_else(|| input.clone());
    Ma((x + prev) / two)
}

fn main() -> anyhow::Result<()> {
    let enable_add_two = std::env::var("ENABLE_ADD_TWO").is_ok();
    let op = insert_env_and_output(ma)
        .inspect(|value, context| {
            println!("input: {}", value);
            if let Some(AddTwo(x)) = context.env().get::<AddTwo<Decimal>>() {
                println!("AddTwo: {x}");
            }
        })
        .insert_env_if(enable_add_two, add_two)
        .cache(1)
        .finish();
    let data = [dec!(1), dec!(2), dec!(3)];

    let ans = data
        .into_iter()
        .indicator(op)
        .map(|Ma(x)| x)
        .collect::<Vec<_>>();

    if enable_add_two {
        assert_eq!(ans, [dec!(1.5), dec!(2.75), dec!(3.875)]);
    } else {
        assert_eq!(ans, [dec!(0.5), dec!(1.25), dec!(2.125)]);
    }
    Ok(())
}
