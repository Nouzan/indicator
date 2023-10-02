use indicator::{prelude::*, IndicatorIteratorExt};

struct Count(usize);

/// Odds counter.
#[operator(i32)]
fn odds_counter(In(value): In<&i32>, Data(count): Data<Option<&Count>>) -> Option<Count> {
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

/// Even signal.
#[operator(i32)]
fn even_signal(
    In(input): In<&i32>,
    Data(count): Data<Option<&EvenCount>>,
) -> (bool, Option<EvenCount>) {
    let count = count.map(|c| c.0).unwrap_or(0);
    if *input % 2 == 0 {
        (true, Some(EvenCount(count + 1)))
    } else {
        (false, if count == 0 { Some(EvenCount(0)) } else { None })
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
        .insert_data(odds_counter)
        .insert_with_data(even_signal)
        .cache(1)
        .finish();
    let data = [1, 2, 3];

    assert_eq!(
        data.into_iter().indicator(op).collect::<Vec<_>>(),
        [false, true, false]
    );
    Ok(())
}
