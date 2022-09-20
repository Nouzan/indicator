/// The definition of the operator of indicators and some helpers
pub mod operator;

pub use operator::{identity::id, map::map, Operator, OperatorExt};
