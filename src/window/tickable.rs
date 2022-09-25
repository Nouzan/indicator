use super::tick_value::TickValue;
use super::Tick;

/// Type that has tick.
pub trait Tickable {
    /// Value.
    type Value;

    /// Get the tick.
    fn tick(&self) -> Tick;

    /// Get the value.
    fn value(&self) -> &Self::Value;

    /// Convert into a [`TickValue`].
    fn into_tick_value(self) -> TickValue<Self::Value>;
}
