/// The definition of the operators of indicators and some helpers
pub mod operator;

/// Helpers for operators with [`Tickable`](crate::Tickable) output.
pub mod tick_operator;

/// Tumbling operator.
pub mod tumbling_operator;

pub use operator::{identity::id, map::map, mux::mux, GatOperator, GatOperatorExt};
pub use tick_operator::{map_tick::map_t, TickGatOperatorExt};
pub use tumbling_operator::{
    operator::{tumbling, view, TumblingOperator},
    periodic::{cache, periodic, periodic_fn, periodic_with, Periodic},
    queue::{circular::Circular, Change, Queue, QueueRef, Tumbling},
};
