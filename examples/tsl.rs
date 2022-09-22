use indicator::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::{macros::datetime, UtcOffset};

fn tr(period: Period) -> impl Operator<TickValue<Decimal>, Output = TickValue<Decimal>> {
    let close = cache::<2, TickValue<Decimal>>(2, period);
    let high = {
        let mut high = Decimal::ZERO;
        view(move |w: QueueRef<TickValue<Decimal>>| {
            if w.change().is_new_period() {
                high = w[0].value;
            } else {
                high = w[0].value.max(high);
            }
            high
        })
    };
    let low = {
        let mut low = Decimal::ZERO;
        view(move |w: QueueRef<TickValue<Decimal>>| {
            if w.change().is_new_period() {
                low = w[0].value;
            } else {
                low = w[0].value.min(low);
            }
            low
        })
    };
    close
        .then(
            view(|w: QueueRef<TickValue<Decimal>>| {
                let close1 = w.get(1).map(|t| t.value).unwrap_or_default();
                w[0].tick.with_value(close1)
            })
            .mux_with(high)
            .mux_with(low),
        )
        .map(|((close1, high), low)| {
            close1.map(|close1| {
                (high - low)
                    .max((close1 - high).abs())
                    .max((close1 - low).abs())
            })
        })
        .into_operator()
}

fn main() {
    let period = Period::seconds(UtcOffset::UTC, 2);
    let mut op = tr(period);

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
