use super::tick::Tick;
use super::tickable::Tickable;

/// Value with timestamp.
pub struct TickValue<T> {
    /// Tick.
    pub tick: Tick,
    /// Value.
    pub value: T,
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
