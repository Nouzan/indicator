#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use indicator::{
    context::{extractor::In, layer::insert::Insert, output_with, ContextOperatorExt},
    operator, IndicatorIteratorExt,
};
use num::Num;
use rust_decimal_macros::dec;

/// A foo operator.
///
/// Just add two to the input, and return the result twice.
#[operator(T)]
fn add_two<T: Num + Clone>(In(value): In<&T>) -> (T, T) {
    (
        value.clone() + T::one() + T::one(),
        value.clone() + T::one() + T::one(),
    )
}

fn main() -> anyhow::Result<()> {
    let op = output_with(add_two()).with(Insert(add_two())).finish();
    let data = [dec!(1), dec!(2), dec!(3)];

    assert_eq!(
        data.into_iter().indicator(op).collect::<Vec<_>>(),
        [(dec!(3), dec!(3)), (dec!(4), dec!(4)), (dec!(5), dec!(5)),]
    );
    Ok(())
}
