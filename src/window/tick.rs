#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::cmp::{Ord, Ordering, PartialOrd};
use time::OffsetDateTime;

use crate::TickValue;

#[cfg(feature = "serde")]
mod rfc3339 {
    use serde::{Deserializer, Serializer};
    use time::{serde::rfc3339, OffsetDateTime};

    /// Serialize an [`Option<OffsetDateTime>`] using the well-known RFC3339 format.
    pub(super) fn serialize<S: Serializer>(
        datetime: &Option<OffsetDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match datetime {
            Some(datetime) => rfc3339::serialize(datetime, serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize an [`OffsetDateTime`] from its RFC3339 representation.
    pub(super) fn deserialize<'a, D: Deserializer<'a>>(
        deserializer: D,
    ) -> Result<Option<OffsetDateTime>, D::Error> {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Option<OffsetDateTime>;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "a optional timestamp str in RFC3339 format")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(Some(rfc3339::deserialize(deserializer)?))
            }
        }

        deserializer.deserialize_option(Visitor)
    }
}

/// A tick in time.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick(#[cfg_attr(feature = "serde", serde(with = "rfc3339"))] Option<OffsetDateTime>);

impl Tick {
    /// "The Big Bang" tick.
    pub const BIG_BANG: Tick = Tick(None);

    /// Create a new tick.
    pub fn new(ts: OffsetDateTime) -> Self {
        Self(Some(ts))
    }

    /// Get the timestamp.
    pub fn ts(&self) -> Option<&OffsetDateTime> {
        self.0.as_ref()
    }

    /// With value.
    pub fn with_value<T>(self, value: T) -> TickValue<T> {
        TickValue { tick: self, value }
    }
}

impl From<OffsetDateTime> for Tick {
    fn from(value: OffsetDateTime) -> Self {
        Self::new(value)
    }
}

impl PartialOrd for Tick {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tick {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.as_ref(), other.0.as_ref()) {
            (Some(lhs), Some(rhs)) => lhs.cmp(rhs),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_bang() {
        assert_eq!(Tick::BIG_BANG, Tick::BIG_BANG);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_tick() {
        use time::macros::datetime;

        let tick = Tick::BIG_BANG;
        assert_eq!(serde_json::to_string(&tick).unwrap(), r#"null"#);

        let tick = Tick::new(datetime!(2022-09-26 01:23:45 +06:54));
        assert_eq!(
            serde_json::to_string(&tick).unwrap(),
            r#""2022-09-26T01:23:45+06:54""#
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_tick() {
        use time::macros::datetime;

        let tick = serde_json::from_str::<Tick>(r#"null"#).unwrap();
        assert_eq!(tick, Tick::BIG_BANG);

        let tick = serde_json::from_str::<Tick>(r#""2022-09-26T01:23:45+06:54""#).unwrap();
        assert_eq!(tick, Tick::new(datetime!(2022-09-26 01:23:45 +06:54)),);
    }
}
