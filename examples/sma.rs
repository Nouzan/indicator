use indicator::{gat::*, Period, Tick, TickValue, TumblingWindow};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::{macros::datetime, UtcOffset};

fn main() {
    let period = Period::seconds(UtcOffset::UTC, 2);
    let mut last_tick = Tick::BIG_BANG;
    let cache = tumbling::<1, TickValue<_>, _, _>(3, move |w, x| {
        if period.same_window(&last_tick, &x.tick) {
            w.swap(x.value);
        } else {
            last_tick = x.tick;
            w.push(x.value);
        };
    });

    let mut sms = Decimal::ZERO;
    let mut op = cache.map(|w| {
        let outdated = w.change().outdated().copied().unwrap_or_default();
        debug_assert!(w.len() != 0);
        sms += w[0] - outdated;
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
