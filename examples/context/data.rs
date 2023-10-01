use indicator::{prelude::*, IndicatorIteratorExt};
use num::Num;
use rust_decimal::Decimal;

#[derive(Clone)]
struct AddTwo<T>(T);

/// An operator that just add two to the input.
#[operator(T)]
fn add_two<T>(In(value): In<&T>, Data(data): Data<&&str>) -> AddTwo<T>
where
    T: Num + Clone,
    T: Send + Sync + 'static,
{
    let two = T::one() + T::one();
    println!("add_two: {data}");
    AddTwo(value.clone() + two)
}

struct Count(usize);

/// Odds counter.
#[operator(i32)]
fn odds_counter(In(value): In<&i32>, Data(count): Data<Option<&Count>>) -> Option<Count> {
    let count = count.map(|c| c.0).unwrap_or(0);
    if *value % 2 == 1 {
        Some(Count(count + 1))
    } else {
        None
    }
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
            let count = value.context.data().get::<Count>().unwrap();
            println!("odds count: {}", count.0);
        })
        .from_context::<&str>() // Asserting that the context has a `&str` data.
        .provide("This is my data!")
        .insert_data(odds_counter)
        .cache(1)
        .finish();
    let data = [1, 2, 3];

    assert_eq!(
        data.into_iter()
            .indicator(op)
            .map(|AddTwo(x)| x)
            .collect::<Vec<_>>(),
        [3, 4, 5]
    );
    Ok(())
}
