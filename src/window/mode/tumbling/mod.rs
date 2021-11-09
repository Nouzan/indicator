/// Period mode.
pub mod period;

/// New mode.
pub mod new;

use super::super::Tick;
use super::WindowMode;
pub use period::Period;

/// Tumbling window mode.
pub trait TumblingWindow: WindowMode {
    /// Is in the same window.
    fn same_window(&self, lhs: &Tick, rhs: &Tick) -> bool;
}

impl<M: TumblingWindow + Clone> WindowMode for M {}
