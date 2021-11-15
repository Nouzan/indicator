use arrayvec::ArrayVec;
use indicator::{facet_t, map_t, tumbling, IndicatorIteratorExt, OperatorExt, Period, TickValue};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::macros::{datetime, offset};

fn main() {
    let data = vec![
        (datetime!(2021-11-01 00:00:00 +0), dec!(199.74)),
        (datetime!(2021-11-01 00:15:00 +0), dec!(198.8)),
        (datetime!(2021-11-01 00:32:00 +0), dec!(200.3)),
        (datetime!(2021-11-01 00:59:59 +0), dec!(178.2)),
        (datetime!(2021-11-01 01:00:00 +0), dec!(201.2)),
        (datetime!(2021-11-01 01:15:00 +0), dec!(203.8)),
        (datetime!(2021-11-01 01:33:00 +0), dec!(193.3)),
        (datetime!(2021-11-01 01:57:59 +0), dec!(200.2)),
        (datetime!(2021-11-01 02:10:00 +0), dec!(205.2)),
        (datetime!(2021-11-01 02:31:00 +0), dec!(193.7)),
        (datetime!(2021-11-01 02:53:59 +0), dec!(201.1)),
    ];
    let op = tumbling(
        Period::hours(offset!(+0), 1),
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
    .map(|v| v.value);
    for ohlc in data
        .into_iter()
        .map(|(ts, value)| TickValue::new(ts, value))
        .indicator(op)
    {
        println!("{:?}", ohlc);
    }
}
