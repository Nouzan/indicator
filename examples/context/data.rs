use indicator::{prelude::*, IndicatorIteratorExt};

struct Count(usize);

/// Odds counter.
#[operator(input = i32)]
fn odds_counter(In(value): In<&i32>, Data(count): Data<Option<&EvenCount>>) -> Option<Count> {
    let count = count.map(|c| c.0).unwrap_or(0);
    if *value % 2 == 1 {
        Some(Count(count + 1))
    } else if count == 0 {
        Some(Count(0))
    } else {
        None
    }
}

struct EvenCount(usize);

impl From<usize> for EvenCount {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Even signal.
#[operator(input = i32, generate_out_with_data)]
fn even_signal(
    In(input): In<&i32>,
    Data(count): Data<Option<&EvenCount>>,
) -> (bool, Option<usize>) {
    let count = count.map(|c| c.0).unwrap_or(0);
    if *input % 2 == 0 {
        (true, Some(count + 1))
    } else {
        (false, if count == 0 { Some(0) } else { None })
    }
}

fn main() -> anyhow::Result<()> {
    let op = output(|_, ctx| ctx.env().get::<bool>().copied().unwrap())
        .inspect(|value, context| {
            println!("input: {}", value);
            if let Some(data) = context.data().get::<&str>() {
                println!("data: {}", data);
            }
            if let Some(count) = context.data().get::<Count>() {
                println!("previous odds count: {}", count.0);
            }
            if let Some(count) = context.data().get::<EvenCount>() {
                println!("previous even count: {}", count.0);
            }
        })
        .from_context::<&str>() // Asserting that the context has a `&str` data.
        .provide("This is my data!")
        .insert_data(odds_counter)
        .insert(even_signal::<bool, EvenCount>)
        .cache(1)
        .finish();
    let data = [1, 2, 3];

    assert_eq!(
        data.into_iter().indicator(op).collect::<Vec<_>>(),
        [false, true, false]
    );
    Ok(())
}
