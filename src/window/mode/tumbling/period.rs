use super::Tick;
use super::TumblingWindow;
use core::hash::Hash;
use time::{Duration, OffsetDateTime, UtcOffset};

/// Period kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PeriodKind {
    /// Zero.
    Zero,
    /// A year.
    Year,
    /// A month.
    Month,
    /// A day.
    Day,
    /// Duration.
    Duration(Duration),
}

/// Period mode (A tumbling window).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Period {
    offset: UtcOffset,
    kind: PeriodKind,
}

impl Period {
    /// Zero period.
    ///
    /// Directly compare the two timestamp.
    pub fn zero() -> Self {
        Self {
            offset: UtcOffset::UTC,
            kind: PeriodKind::Zero,
        }
    }

    /// A year.
    pub fn year(offset: UtcOffset) -> Self {
        Self {
            offset,
            kind: PeriodKind::Year,
        }
    }

    /// A Month.
    pub fn month(offset: UtcOffset) -> Self {
        Self {
            offset,
            kind: PeriodKind::Month,
        }
    }

    /// A Day.
    pub fn day(offset: UtcOffset) -> Self {
        Self {
            offset,
            kind: PeriodKind::Day,
        }
    }

    /// Weeks.
    pub fn weeks(offset: UtcOffset, weeks: u32) -> Self {
        if weeks == 0 {
            Self::zero()
        } else {
            Self {
                offset,
                kind: PeriodKind::Duration(Duration::weeks(weeks as i64)),
            }
        }
    }

    /// Days.
    pub fn days(offset: UtcOffset, days: u32) -> Self {
        match days {
            0 => Self::zero(),
            1 => Self::day(offset),
            days => Self {
                offset,
                kind: PeriodKind::Duration(Duration::days(days as i64)),
            },
        }
    }

    /// Hours.
    pub fn hours(offset: UtcOffset, hours: u32) -> Self {
        if hours == 0 {
            Self::zero()
        } else {
            Self {
                offset,
                kind: PeriodKind::Duration(Duration::hours(hours as i64)),
            }
        }
    }

    /// Minutes
    pub fn minutes(offset: UtcOffset, minutes: u32) -> Self {
        if minutes == 0 {
            Self::zero()
        } else {
            Self {
                offset,
                kind: PeriodKind::Duration(Duration::minutes(minutes as i64)),
            }
        }
    }

    /// Seconds
    pub fn seconds(offset: UtcOffset, seconds: u32) -> Self {
        if seconds == 0 {
            Self::zero()
        } else {
            Self {
                offset,
                kind: PeriodKind::Duration(Duration::seconds(seconds as i64)),
            }
        }
    }

    /// Convert period to [`Duration`].
    ///
    /// Return `None` if period is a year or a month.
    pub fn to_duration(&self) -> Option<Duration> {
        match self.kind {
            PeriodKind::Zero => Some(Duration::ZERO),
            PeriodKind::Year | PeriodKind::Month => None,
            PeriodKind::Day => Some(Duration::DAY),
            PeriodKind::Duration(d) => Some(d),
        }
    }
}

const WEEK_OFFSET: Duration = Duration::days(4);

impl TumblingWindow for Period {
    fn same_window(&self, lhs: &Tick, rhs: &Tick) -> bool {
        let lhs = lhs.ts().map(|t| t.to_offset(self.offset));
        let rhs = rhs.ts().map(|t| t.to_offset(self.offset));
        match (lhs, rhs) {
            (Some(lhs), Some(rhs)) => match self.kind {
                PeriodKind::Zero => lhs == rhs,
                PeriodKind::Year => lhs.year() == rhs.year(),
                PeriodKind::Month => lhs.year() == rhs.year() && lhs.month() == rhs.month(),
                PeriodKind::Day => lhs.date() == rhs.date(),
                PeriodKind::Duration(d) => {
                    let d = d.whole_seconds();
                    if d == 0 {
                        return lhs == rhs;
                    }
                    let base = OffsetDateTime::UNIX_EPOCH.replace_offset(self.offset) + WEEK_OFFSET;
                    let lhs = (lhs - base).whole_seconds() / d;
                    let rhs = (rhs - base).whole_seconds() / d;
                    lhs == rhs
                }
            },
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Period, TumblingWindow};
    use time::macros::{datetime, offset};
    use time::UtcOffset;

    #[test]
    fn zero_period() {
        let mode = Period::zero();
        let lhs = datetime!(2021-11-1 00:00:00 +08).into();
        let rhs = datetime!(2021-11-1 00:00:00 +08).into();
        assert!(mode.same_window(&lhs, &rhs));
    }

    #[test]
    fn week_different_utc_offset() {
        let lhs = datetime!(2021-11-1 00:00:00 +08).into();
        let rhs = datetime!(2021-11-1 00:00:00 UTC).into();
        let mode = Period::weeks(UtcOffset::UTC, 1);
        assert!(!mode.same_window(&lhs, &rhs));
        let mode = Period::weeks(offset!(+8), 1);
        assert!(mode.same_window(&lhs, &rhs));
    }

    #[test]
    fn week() {
        let mode = Period::weeks(offset!(+8), 1);
        let lhs = datetime!(2021-11-1 07:00:12 +08).into();
        let rhs = datetime!(2021-11-7 12:00:21 +08).into();
        assert!(mode.same_window(&lhs, &rhs));
        let rhs = datetime!(2021-11-8 1:00:12 + 08).into();
        assert!(!mode.same_window(&lhs, &rhs));
    }

    #[test]
    fn hours() {
        let mode = Period::hours(offset!(+8), 2);
        let lhs = datetime!(2021-11-1 00:00:00 +08).into();
        let rhs = datetime!(2021-11-1 01:29:31 +08).into();
        assert!(mode.same_window(&lhs, &rhs));
        let rhs = datetime!(2021-11-1 02:00:00 +08).into();
        assert!(!mode.same_window(&lhs, &rhs));
        let rhs = datetime!(2021-10-31 23:59:59 +08).into();
        assert!(!mode.same_window(&lhs, &rhs));
    }
}
