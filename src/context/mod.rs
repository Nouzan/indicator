mod anymap;

/// Input with context.
pub mod input;

/// Convert an `In`-operator to another `In`-operator.
pub mod layer;

use crate::Operator;

pub use self::{
    anymap::Context,
    input::{input, In, Input},
    layer::Layer,
};

/// `In`-operator.
/// Alias for `Operator<In<T>>`.
pub trait InOperator<T>: Operator<In<T>> {
    /// Apply a layer.
    fn with<L>(self, layer: L) -> L::Output
    where
        L: Layer<T, Self>,
        Self: Sized,
    {
        layer.layer(self)
    }
}

impl<T, P> InOperator<T> for P where P: Operator<In<T>> {}
