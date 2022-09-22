use core::cmp::{Ord, Ordering, PartialOrd};
#[cfg(feature = "serde-derive")]
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::TickValue;

#[cfg(not(feature = "serde-derive"))]
/// A tick in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick(Option<OffsetDateTime>);

#[cfg(feature = "serde-derive")]
/// A tick in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Tick(Option<OffsetDateTime>);

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
        match (self.0.as_ref(), other.0.as_ref()) {
            (Some(lhs), Some(rhs)) => Some(lhs.cmp(rhs)),
            (Some(_), None) => Some(Ordering::Greater),
            (None, Some(_)) => Some(Ordering::Less),
            (None, None) => Some(Ordering::Equal),
        }
    }
}

impl Ord for Tick {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
