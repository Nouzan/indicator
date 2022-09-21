/// The definition of the operators of indicators and some helpers
pub mod operator;

/// Helpers for operators with [`Tickable`](crate::Tickable) output.
pub mod tick_operator;

pub use operator::{identity::id, map::map, mux::mux, GatOperator, GatOperatorExt};
pub use tick_operator::{map_tick::map_t, TickGatOperatorExt};
