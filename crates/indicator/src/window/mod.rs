/// Timestamp with some window mode.
pub mod tick;

/// Window mod.
pub mod mode;

/// Tickable.
pub mod tickable;

/// Value with timestamp.
pub mod tick_value;

pub use mode::tumbling::{Period, PeriodKind, TumblingWindow};
pub use tick::Tick;
pub use tick_value::TickValue;
pub use tickable::Tickable;
