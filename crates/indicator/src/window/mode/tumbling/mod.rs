/// Period mode.
pub mod period;

/// New mode.
pub mod new;

use super::super::Tick;
pub use period::{Period, PeriodKind};

/// Tumbling window mode.
pub trait TumblingWindow: Clone {
    /// Is in the same window.
    fn same_window(&self, lhs: &Tick, rhs: &Tick) -> bool;
}
