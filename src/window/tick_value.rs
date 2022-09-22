use super::tick::Tick;
use super::tickable::Tickable;
use time::OffsetDateTime;

/// Value with timestamp.
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

    fn tick(&self) -> &Tick {
        &self.tick
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
