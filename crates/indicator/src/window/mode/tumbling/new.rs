use super::Tick;
use super::TumblingWindow;

/// New mode.
///
/// Every two tick will not in the same window.
#[derive(Debug, Clone, Copy)]
pub struct New;

impl TumblingWindow for New {
    fn same_window(&self, _lhs: &Tick, _rhs: &Tick) -> bool {
        false
    }
}
