# Indicator

Abstractions for stream aggregation, we call them `Indicator` s.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/indicator.svg
[crates-url]: https://crates.io/crates/indicator
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/Nouzan/indicator/blob/master/LICENSE
[actions-badge]: https://github.com/Nouzan/indicator/workflows/CI/badge.svg
[actions-url]: https://github.com/Nouzan/indicator/actions?query=workflow%3ACI+branch%3Amain

[API Docs](https://docs.rs/indicator/latest/indicator)

## Example

Add `indicator` as a dependency of your project.

```toml
[dependencies]
indicator = "0.3"
rust_decimal = "1.17.0"
rust_decimal_macros = "1.17.0"
time = { version = "0.3", default-features = false, features = ["macros"] }
```

And then, you can try these codes.

```rust
use arrayvec::ArrayVec;
use indicator::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::macros::{datetime, offset};

/// Return an indicator that calculate `hl2` and `ohlc4` simultaneously.
fn hl2_ohlc4(period: Period) -> impl Operator<TickValue<Decimal>, Output = (Decimal, Decimal)> {
    tumbling(
        period,
        |_w: &ArrayVec<[Decimal; 4], 0>, y: &mut Option<[Decimal; 4]>, x| match y {
            Some(ohlc) => {
                ohlc[1] = ohlc[1].max(x);
                ohlc[2] = ohlc[2].min(x);
                ohlc[3] = x;
                *ohlc
            }
            None => {
                let ohlc = [x; 4];
                *y = Some(ohlc);
                ohlc
            }
        },
    )
    .then(facet_t(
        map_t(|ohlc: [Decimal; 4]| (ohlc[1] + ohlc[2]) / dec!(2)),
        map_t(|ohlc: [Decimal; 4]| (ohlc[0] + ohlc[1] + ohlc[2] + ohlc[3]) / dec!(4)),
    ))
    .map(|v| v.value)
}
```
