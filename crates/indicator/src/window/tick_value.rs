use core::borrow::Borrow;
use core::ops::Deref;

use super::tick::Tick;
use super::tickable::Tickable;
use time::OffsetDateTime;

/// Value with timestamp.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickValue<T> {
    /// Tick.
    pub tick: Tick,
    /// Value.
    pub value: T,
}

impl<T> TickValue<T> {
    /// Create a new [`TickValue`] with the given `value`
    /// and the `BIG_BANG` tick.
    pub fn big_bang(value: T) -> Self {
        Self {
            tick: Tick::BIG_BANG,
            value,
        }
    }

    /// Create a new [`TickValue`] from `ts` and `value`.
    pub fn new(ts: OffsetDateTime, value: T) -> Self {
        Self {
            tick: Tick::new(ts),
            value,
        }
    }

    /// Map over the tick value.
    pub fn map<U, F>(self, f: F) -> TickValue<U>
    where
        F: FnOnce(T) -> U,
    {
        TickValue {
            tick: self.tick,
            value: (f)(self.value),
        }
    }
}

impl<T> Tickable for TickValue<T> {
    type Value = T;

    fn tick(&self) -> Tick {
        self.tick
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn into_tick_value(self) -> TickValue<Self::Value> {
        self
    }
}

impl<T> core::fmt::Display for TickValue<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(ts) = self.tick.ts() {
            write!(f, "({ts}, {})", self.value)
        } else {
            write!(f, "(*, {})", self.value)
        }
    }
}

impl<T> Deref for TickValue<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> Borrow<T> for TickValue<T> {
    fn borrow(&self) -> &T {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_value() {
        let tick_value = TickValue::new(OffsetDateTime::UNIX_EPOCH, 42);
        assert_eq!(tick_value.tick(), Tick::new(OffsetDateTime::UNIX_EPOCH));
        assert_eq!(tick_value.value(), &42);
        assert_eq!(tick_value.into_tick_value(), tick_value);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_tick_value() {
        use time::macros::datetime;

        let tick_value = TickValue::new(datetime!(2021-01-01 00:00:00 UTC), 42);
        let json = serde_json::to_string(&tick_value).unwrap();
        assert_eq!(json, r#"{"tick":"2021-01-01T00:00:00Z","value":42}"#);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_tick_value() {
        use time::macros::datetime;

        let json = r#"{"tick":"2021-01-01T00:00:00Z","value":42}"#;
        let tick_value: TickValue<i32> = serde_json::from_str(json).unwrap();
        assert_eq!(
            tick_value.tick(),
            Tick::new(datetime!(2021-01-01 00:00:00 UTC))
        );
        assert_eq!(tick_value.value(), &42);
    }
}
