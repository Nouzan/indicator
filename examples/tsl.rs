use indicator::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::{macros::datetime, UtcOffset};

fn tsl(
    length: Decimal,
    factor: Decimal,
    period: Period,
) -> impl Operator<TickValue<Decimal>, Output = TickValue<Decimal>> {
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

    let cache0 = view(|w: QueueRef<TickValue<Decimal>>| w[0]);

    let cache1 = view(|w: QueueRef<TickValue<Decimal>>| {
        let close1 = w.get(1).map(|t| t.value);
        w[0].tick.with_value(close1)
    });

    // rma = (1 - 1/l) * x + 1/l * rma[1]
    let alpha = Decimal::ONE / length;
    let rma = periodic::<2, _, _, _>(
        2,
        period,
        periodic_fn(
            move |w: QueueRef<TickValue<Decimal>>, x: TickValue<Decimal>, _n| {
                let rma1 = w.get(1).map(|t| t.value).unwrap_or(x.value);
                x.tick
                    .with_value((Decimal::ONE - alpha) * x.value + alpha * rma1)
            },
        ),
    );

    let atr = map(
        |(((_close0, close1), high), low): (
            ((TickValue<Decimal>, TickValue<Option<Decimal>>), Decimal),
            Decimal,
        )| {
            close1.map(|close1| {
                close1
                    .map(|close1| {
                        (high - low)
                            .max((close1 - high).abs())
                            .max((close1 - low).abs())
                    })
                    .unwrap_or_default()
            })
        },
    )
    .then(rma)
    .then(view(|w| w[0]));

    // long = true if last >= tsl[1] && !long[1]
    //        false if last <= tsl[1] && long[1]
    //        long[1] otherwise
    // tsl = down if last >= tsl[1] && !long[1]
    //       up if last <= tsl[1] && long[1]
    //       max(tsl[1], down) if long[1]
    //       min(tsl[1], up) if !long[1]
    let tsl = periodic::<2, _, _, _>(
        2,
        period,
        periodic_fn(
            |w: QueueRef<TickValue<(Decimal, bool)>>,
             x: TickValue<(Decimal, Decimal, Decimal)>,
             _n| {
                if let Some(tsl1) = w.get(1) {
                    let TickValue {
                        tick,
                        value: (last, up, down),
                    } = x;
                    let long1 = tsl1.value.1;
                    let tsl1 = tsl1.value.0;
                    let v = if long1 {
                        let cross = last <= tsl1;
                        let tsl = if cross { up } else { tsl1.max(down) };
                        (tsl, !cross)
                    } else {
                        let cross = last >= tsl1;
                        let tsl = if cross { down } else { tsl1.min(up) };
                        (tsl, cross)
                    };
                    tick.with_value(v)
                } else {
                    x.map(|(_, _, down)| (down, true))
                }
            },
        ),
    )
    .then(view(|w: QueueRef<TickValue<(Decimal, bool)>>| {
        w[0].map(|v| v.0)
    }));

    close
        .then(cache0.mux_with(cache1).mux_with(high).mux_with(low))
        .then(id().mux_with(atr))
        .map(move |((((last, _close1), high), low), atr)| {
            let bias = factor * atr.value;
            let up = high + bias;
            let down = low - bias;
            last.map(|last| (last, up, down))
        })
        .then(tsl)
        .into_operator()
}

fn main() {
    let period = Period::seconds(UtcOffset::UTC, 2);
    let mut op = tsl(dec!(3), dec!(1), period);

    for x in [
        TickValue::new(datetime!(2022-09-22 00:00:00 +00:00), dec!(100)),
        TickValue::new(datetime!(2022-09-22 00:00:01 +00:00), dec!(102)),
        TickValue::new(datetime!(2022-09-22 00:00:02 +00:00), dec!(103)),
        TickValue::new(datetime!(2022-09-22 00:00:03 +00:00), dec!(104)),
        TickValue::new(datetime!(2022-09-22 00:00:04 +00:00), dec!(105)),
        TickValue::new(datetime!(2022-09-22 00:00:05 +00:00), dec!(106)),
        TickValue::new(datetime!(2022-09-22 00:00:06 +00:00), dec!(100)),
        TickValue::new(datetime!(2022-09-22 00:00:07 +00:00), dec!(102)),
        TickValue::new(datetime!(2022-09-22 00:00:08 +00:00), dec!(103)),
        TickValue::new(datetime!(2022-09-22 00:00:09 +00:00), dec!(104)),
        TickValue::new(datetime!(2022-09-22 00:00:10 +00:00), dec!(105)),
        TickValue::new(datetime!(2022-09-22 00:00:11 +00:00), dec!(106)),
    ] {
        println!("{}", op.next(x));
    }
}
