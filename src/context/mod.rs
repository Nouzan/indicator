mod anymap;

/// Input with context.
pub mod input;

pub use self::{
    anymap::Context,
    input::{input, In, Input},
};
