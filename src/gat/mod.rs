/// The definition of the operators of indicators and some helpers
pub mod operator;

/// Helpers for operators with [`Tickable`](crate::Tickable) output.
pub mod tick_operator;

pub use operator::{identity::id, map::map, mux::mux, Operator, OperatorExt};
pub use tick_operator::{tick_map::map_t, TickOperatorExt};
