use indicator::{prelude::*, IndicatorIteratorExt};

struct Count(usize);

impl From<usize> for Count {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Odds counter.
#[operator(input = i32, generate_data)]
fn odds_counter(In(value): In<&i32>, Data(count): Data<Option<&Count>>) -> Option<usize> {
    let count = count.map(|c| c.0).unwrap_or(0);
    if *value % 2 == 1 {
        Some(count + 1)
    } else if count == 0 {
        Some(0)
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
        .inspect(|value| {
            println!("input: {}", value.value());
            if let Some(data) = value.context().data().get::<&str>() {
                println!("data: {}", data);
            }
            let count = value.context().data().get::<Count>().unwrap();
            println!("odds count: {}", count.0);
            let count = value.context().data().get::<EvenCount>().unwrap();
            println!("even count: {}", count.0);
        })
        .from_context::<&str>() // Asserting that the context has a `&str` data.
        .provide("This is my data!")
        .insert_data(odds_counter::<Count>)
        .insert_with_data(even_signal::<bool, EvenCount>)
        .cache(1)
        .finish();
    let data = [1, 2, 3];

    assert_eq!(
        data.into_iter().indicator(op).collect::<Vec<_>>(),
        [false, true, false]
    );
    Ok(())
}
