#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use indicator::{prelude::*, IndicatorIteratorExt};
use num::Num;
use rust_decimal_macros::dec;

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

// Derive `Clone` to satisfy the requirement of `output_with`.
#[derive(Clone)]
struct Ma<T>(T);

/// An operator that does the following:
/// `x => (x + prev(x)) / 2`
#[operator(T)]
fn ma<T>(Env(AddTwo(x)): Env<&AddTwo<T>>, Prev(prev): Prev<&Ma<T>>) -> Ma<T>
where
    T: Num + Clone,
    T: Send + Sync + 'static,
{
    let two = T::one() + T::one();
    let prev = prev.map(|v| v.0.clone()).unwrap_or(T::zero());
    Ma((x.clone() + prev) / two)
}

fn main() -> anyhow::Result<()> {
    let op = output_with(ma)
        .with(Insert(add_two))
        .with(Cache::new())
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
