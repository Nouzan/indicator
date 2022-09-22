use indicator::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::{macros::datetime, UtcOffset};

fn main() {
    let period = Period::seconds(UtcOffset::UTC, 2);
    let cache = cache::<3, TickValue<_>>(3, period);
    let mut sms = Decimal::ZERO;
    let mut op = cache.map(|w| {
        assert!(w.is_inline());
        let outdated = w.change().outdated().map(|t| t.value).unwrap_or_default();
        debug_assert!(w.len() != 0);
        sms += w[0].value - outdated;
        let l = Decimal::new(w.len() as i64, 0);
        sms / l
    });

    for x in [
        TickValue::new(datetime!(2022-09-22 00:00:00 +00:00), dec!(1)),
        TickValue::new(datetime!(2022-09-22 00:00:01 +00:00), dec!(2)),
        TickValue::new(datetime!(2022-09-22 00:00:02 +00:00), dec!(3)),
        TickValue::new(datetime!(2022-09-22 00:00:03 +00:00), dec!(4)),
        TickValue::new(datetime!(2022-09-22 00:00:04 +00:00), dec!(5)),
        TickValue::new(datetime!(2022-09-22 00:00:05 +00:00), dec!(6)),
    ] {
        println!("{}", op.next(x));
    }
}
