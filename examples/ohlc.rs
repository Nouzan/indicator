use arrayvec::ArrayVec;
use indicator::{
    facet_t, map_t, tumbling, IndicatorIteratorExt, Operator, OperatorExt, Period, TickValue,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::macros::{datetime, offset};

fn indicator(period: Period) -> impl Operator<TickValue<Decimal>, Output = (Decimal, Decimal)> {
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

fn main() {
    let data = vec![
        (datetime!(2021-11-01 00:00:00 +0), dec!(2)),
        (datetime!(2021-11-01 00:15:00 +0), dec!(1)),
        (datetime!(2021-11-01 00:32:00 +0), dec!(4)),
        (datetime!(2021-11-01 00:59:59 +0), dec!(3)),
        (datetime!(2021-11-01 01:00:00 +0), dec!(3)),
        (datetime!(2021-11-01 01:15:00 +0), dec!(4)),
        (datetime!(2021-11-01 01:33:00 +0), dec!(2)),
        (datetime!(2021-11-01 01:57:59 +0), dec!(3)),
        (datetime!(2021-11-01 02:10:00 +0), dec!(1)),
        (datetime!(2021-11-01 02:31:00 +0), dec!(2)),
        (datetime!(2021-11-01 02:53:59 +0), dec!(3)),
    ];
    let period = Period::hours(offset!(+0), 1);
    let op = indicator(period);
    for ohlc in data
        .into_iter()
        .map(|(ts, value)| TickValue::new(ts, value))
        .indicator(op)
    {
        println!("{:?}", ohlc);
    }
}
