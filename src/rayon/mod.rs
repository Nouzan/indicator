/// Normal combinator.
pub mod normal;

/// Ticked combinator.
pub mod ticked;

pub use normal::{facet, facet_map};
pub use ticked::{facet_map_t, facet_t};
