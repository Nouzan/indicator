use indicator::{prelude::*, IndicatorIteratorExt};
use num::Num;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Clone)]
struct AddTwo<T>(T);

/// An operator that just add two to the input.
#[operator(T)]
fn add_two<T>(In(value): In<&T>) -> AddTwo<T>
where
    T: Num + Clone,
    T: Send + Sync + 'static,
{
    let two = T::one() + T::one();
    AddTwo(value.clone() + two)
}

fn main() -> anyhow::Result<()> {
    let op = output_with(add_two)
        .inspect(|value| {
            println!("input: {}", value.value);
            if let Some(AddTwo(x)) = value.context.env().get::<AddTwo<Decimal>>() {
                println!("AddTwo: {x}");
            }
            if let Some(data) = value.context.data().get::<&str>() {
                println!("data: {}", data);
            }
        })
        .from_context::<&str>() // Asserting that the context has a `&str` data.
        .provide("This is my data!")
        .cache(1)
        .finish();
    let data = [dec!(1), dec!(2), dec!(3)];

    assert_eq!(
        data.into_iter()
            .indicator(op)
            .map(|AddTwo(x)| x)
            .collect::<Vec<_>>(),
        [dec!(3), dec!(4), dec!(5)]
    );
    Ok(())
}
