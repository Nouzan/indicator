use super::tick::Tick;
use super::tickable::Tickable;
use time::OffsetDateTime;

/// Value with timestamp.
#[derive(Debug, Clone, Copy)]
pub struct TickValue<T> {
    /// Tick.
    pub tick: Tick,
    /// Value.
    pub value: T,
}

impl<T> TickValue<T> {
    /// Create a new [`TickValue`] from timestamp and value.
    pub fn new(ts: OffsetDateTime, value: T) -> Self {
        Self {
            tick: Tick::new(ts),
            value,
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
