use super::Tick;
use super::TumblingWindow;
use core::{cmp::Ordering, fmt, hash::Hash, time::Duration};
use time::{OffsetDateTime, UtcOffset};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Period kind.
#[derive(Debug, Clone, Copy)]
pub enum PeriodKind {
    /// A year.
    Year,
    /// A month.
    Month,
    /// Duration.
    Duration(Duration),
}

impl PartialEq for PeriodKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PeriodKind::Year, PeriodKind::Year) => true,
            (PeriodKind::Month, PeriodKind::Month) => true,
            (PeriodKind::Duration(lhs), PeriodKind::Duration(rhs)) => lhs.eq(rhs),
            _ => false,
        }
    }
}

impl Hash for PeriodKind {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Year => {
                state.write(&[0x00]);
                state.finish();
            }
            Self::Month => {
                state.write(&[0x01]);
                state.finish();
            }
            Self::Duration(d) => {
                state.write(&[0x02]);
                d.hash(state);
            }
        }
    }
}

impl Eq for PeriodKind {}

const YEAD_SECS_LOWER: u64 = 31_536_000;
const YEAD_SECS_UPPER: u64 = 31_622_400;
const MONTH_SECS_LOWER: u64 = 2_419_200;
const MONTH_SECS_UPPER: u64 = 2_678_400;
const DAY_SECS: u64 = 86_400;
const WEEK_SECS: u64 = 604_800;
const HOUR_SECS: u64 = 3_600;
const MINUTE_SECS: u64 = 60;

impl PartialOrd for PeriodKind {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (PeriodKind::Year, PeriodKind::Year) => Some(Ordering::Equal),
            (PeriodKind::Month, PeriodKind::Month) => Some(Ordering::Equal),
            (PeriodKind::Duration(lhs), PeriodKind::Duration(rhs)) => lhs.partial_cmp(rhs),
            (PeriodKind::Year, PeriodKind::Duration(d)) => {
                if d.as_secs() < YEAD_SECS_LOWER {
                    Some(Ordering::Greater)
                } else if d.as_secs() > YEAD_SECS_UPPER {
                    Some(Ordering::Less)
                } else if d.as_secs() == YEAD_SECS_UPPER {
                    if d.subsec_micros() > 0 {
                        Some(Ordering::Less)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            (PeriodKind::Duration(d), PeriodKind::Year) => {
                if d.as_secs() < YEAD_SECS_LOWER {
                    Some(Ordering::Less)
                } else if d.as_secs() > YEAD_SECS_UPPER {
                    Some(Ordering::Greater)
                } else if d.as_secs() == YEAD_SECS_UPPER {
                    if d.subsec_micros() > 0 {
                        Some(Ordering::Greater)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            (PeriodKind::Month, PeriodKind::Duration(d)) => {
                if d.as_secs() < MONTH_SECS_LOWER {
                    Some(Ordering::Greater)
                } else if d.as_secs() > MONTH_SECS_UPPER {
                    Some(Ordering::Less)
                } else if d.as_secs() == MONTH_SECS_UPPER {
                    if d.subsec_micros() > 0 {
                        Some(Ordering::Less)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            (PeriodKind::Duration(d), PeriodKind::Month) => {
                if d.as_secs() < MONTH_SECS_LOWER {
                    Some(Ordering::Less)
                } else if d.as_secs() > MONTH_SECS_UPPER {
                    Some(Ordering::Greater)
                } else if d.as_secs() == MONTH_SECS_UPPER {
                    if d.subsec_micros() > 0 {
                        Some(Ordering::Greater)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            (PeriodKind::Month, PeriodKind::Year) => Some(Ordering::Less),
            (PeriodKind::Year, PeriodKind::Month) => Some(Ordering::Greater),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for PeriodKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Year => serializer.serialize_str("year"),
            Self::Month => serializer.serialize_str("month"),
            Self::Duration(dur) => {
                let formatted = humantime::format_duration(*dur);
                serializer.collect_str(&formatted)
            }
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for PeriodKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PeriodKind;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "\"year\", \"month\" or any `str` that is parsable by `humantime::parse_duration`")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let kind = match v {
                    "year" => PeriodKind::Year,
                    "month" => PeriodKind::Month,
                    v => {
                        let dur = humantime::parse_duration(v)
                            .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))?;
                        PeriodKind::Duration(dur)
                    }
                };
                Ok(kind)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(feature = "serde")]
fn default_offset() -> UtcOffset {
    UtcOffset::UTC
}

/// Period mode (A tumbling window).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Period {
    /// UTC offset.
    #[cfg_attr(feature = "serde", serde(default = "default_offset"))]
    pub offset: UtcOffset,
    /// Period kind.
    #[cfg_attr(feature = "serde", serde(rename = "period"))]
    pub kind: PeriodKind,
}

impl PartialOrd for Period {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.offset.eq(&other.offset) {
            self.kind.partial_cmp(&other.kind)
        } else {
            None
        }
    }
}

impl Period {
    /// Zero period.
    ///
    /// Directly compare the two timestamp.
    pub fn zero() -> Self {
        Self {
            offset: UtcOffset::UTC,
            kind: PeriodKind::Duration(Duration::ZERO),
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
            kind: PeriodKind::Duration(Duration::from_secs(DAY_SECS)),
        }
    }

    /// Weeks.
    pub fn weeks(offset: UtcOffset, weeks: u32) -> Self {
        if weeks == 0 {
            Self::zero()
        } else {
            Self {
                offset,
                kind: PeriodKind::Duration(Duration::from_secs(weeks as u64 * WEEK_SECS)),
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
                kind: PeriodKind::Duration(Duration::from_secs(days as u64 & DAY_SECS)),
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
                kind: PeriodKind::Duration(Duration::from_secs(hours as u64 * HOUR_SECS)),
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
                kind: PeriodKind::Duration(Duration::from_secs(minutes as u64 * MINUTE_SECS)),
            }
        }
    }

    /// Seconds
    pub fn seconds(offset: UtcOffset, seconds: u64) -> Self {
        if seconds == 0 {
            Self::zero()
        } else {
            Self {
                offset,
                kind: PeriodKind::Duration(Duration::from_secs(seconds)),
            }
        }
    }

    /// Seconds.
    pub fn secs(seconds: u64) -> Self {
        if seconds == 0 {
            Self::zero()
        } else {
            Self {
                offset: UtcOffset::UTC,
                kind: PeriodKind::Duration(Duration::from_secs(seconds)),
            }
        }
    }

    /// Convert period to [`Duration`].
    ///
    /// Return `None` if period is a year or a month.
    pub fn to_std_duration(&self) -> Option<Duration> {
        match self.kind {
            PeriodKind::Year | PeriodKind::Month => None,
            PeriodKind::Duration(d) => Some(d),
        }
    }

    /// Convert period to [`time::Duration`].
    ///
    /// Return `None` if period is a year or a month.
    pub fn to_duration(&self) -> Option<time::Duration> {
        match self.kind {
            PeriodKind::Year | PeriodKind::Month => None,
            PeriodKind::Duration(d) => time::Duration::try_from(d).ok(),
        }
    }

    /// Get the utc offset of this period.
    pub fn utc_offset(&self) -> UtcOffset {
        self.offset
    }

    /// Get period kind.
    pub fn kind(&self) -> PeriodKind {
        self.kind
    }

    /// Change the utc offset.
    /// # Example
    /// ```
    /// use indicator::Period;
    /// use time::macros::offset;
    ///
    /// let period = Period::day(offset!(+0));
    /// assert_eq!(period.to_offset(offset!(+8)), Period::day(offset!(+8)));
    /// ```
    pub fn to_offset(&self, offset: UtcOffset) -> Self {
        Self {
            offset,
            kind: self.kind,
        }
    }
}

const WEEK_OFFSET: Duration = Duration::from_secs(4 * DAY_SECS);

impl TumblingWindow for Period {
    fn same_window(&self, lhs: &Tick, rhs: &Tick) -> bool {
        let lhs = lhs.ts().map(|t| t.to_offset(self.offset));
        let rhs = rhs.ts().map(|t| t.to_offset(self.offset));
        match (lhs, rhs) {
            (Some(lhs), Some(rhs)) => match self.kind {
                PeriodKind::Year => lhs.year() == rhs.year(),
                PeriodKind::Month => lhs.year() == rhs.year() && lhs.month() == rhs.month(),
                PeriodKind::Duration(d) => {
                    let d = d.as_secs() as i128;
                    if d == 0 {
                        return lhs == rhs;
                    }
                    let base = OffsetDateTime::UNIX_EPOCH.replace_offset(self.offset) + WEEK_OFFSET;
                    let lhs = (lhs - base).whole_seconds() as i128 / d;
                    let rhs = (rhs - base).whole_seconds() as i128 / d;
                    lhs == rhs
                }
            },
            _ => false,
        }
    }
}

#[cfg(feature = "humantime")]
use humantime::format_duration;

impl fmt::Display for PeriodKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Year => {
                write!(f, "year")
            }
            Self::Month => {
                write!(f, "month")
            }
            Self::Duration(d) => {
                #[cfg(not(feature = "humantime"))]
                {
                    write!(f, "{}s", d.as_secs())
                }
                #[cfg(feature = "humantime")]
                {
                    write!(f, "{}", format_duration(*d))
                }
            }
        }
    }
}

impl fmt::Display for Period {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.kind, self.offset.whole_hours())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[cfg(feature = "std")]
    #[test]
    fn to_string() {
        let mode = Period::hours(offset!(+8), 2);
        println!("{mode}");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_period_kind() {
        let kind = PeriodKind::Year;
        assert_eq!(serde_json::to_string(&kind).unwrap(), r#""year""#);

        let kind = PeriodKind::Month;
        assert_eq!(serde_json::to_string(&kind).unwrap(), r#""month""#);

        let dur = Duration::from_secs(12346789);
        let kind = PeriodKind::Duration(dur);
        assert_eq!(
            serde_json::to_string(&kind).unwrap(),
            r#""4months 21days 3h 25m 25s""#,
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_period_kind() {
        let kind = serde_json::from_str::<PeriodKind>(r#""year""#).unwrap();
        assert_eq!(kind, PeriodKind::Year);

        let kind = serde_json::from_str::<PeriodKind>(r#""month""#).unwrap();
        assert_eq!(kind, PeriodKind::Month);

        let kind = serde_json::from_str::<PeriodKind>(r#""3m 4s""#).unwrap();
        assert_eq!(kind, PeriodKind::Duration(Duration::from_secs(184)),);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_period() {
        let period = Period::year(UtcOffset::UTC);
        assert_eq!(
            serde_json::to_string(&period).unwrap(),
            r#"{"offset":"+00:00:00","period":"year"}"#
        );

        let period = Period::month(offset!(-01:23:45));
        assert_eq!(
            serde_json::to_string(&period).unwrap(),
            r#"{"offset":"-01:23:45","period":"month"}"#
        );

        let period = Period::hours(UtcOffset::UTC, 3);
        assert_eq!(
            serde_json::to_string(&period).unwrap(),
            r#"{"offset":"+00:00:00","period":"3h"}"#
        );

        let period = Period::minutes(UtcOffset::UTC, 42);
        assert_eq!(
            serde_json::to_string(&period).unwrap(),
            r#"{"offset":"+00:00:00","period":"42m"}"#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_period() {
        let period =
            serde_json::from_str::<Period>(r#"{"offset":"+00:00:00","period":"year"}"#).unwrap();
        assert_eq!(period, Period::year(UtcOffset::UTC),);

        let period =
            serde_json::from_str::<Period>(r#"{"offset":"-01:23:45","period":"month"}"#).unwrap();
        assert_eq!(period, Period::month(offset!(-01:23:45)),);

        let period =
            serde_json::from_str::<Period>(r#"{"offset":"+00:00:00","period":"3h"}"#).unwrap();
        assert_eq!(period, Period::hours(UtcOffset::UTC, 3),);

        let period =
            serde_json::from_str::<Period>(r#"{"offset":"+00:00:00","period":"42m"}"#).unwrap();
        assert_eq!(period, Period::minutes(UtcOffset::UTC, 42),);
    }
}
