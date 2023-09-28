use core::{fmt, marker::PhantomData};

use crate::Operator;

use super::Context;

/// Input with context.
#[derive(Debug)]
pub struct In<T> {
    /// Input value.
    value: T,
    /// Context.
    context: Context,
}

impl<T> In<T> {
    /// Create a new `Input` with the given value.
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            value,
            context: Context::new(),
        }
    }

    /// Get the reference to the input value.
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Get the mutable reference to the input value.
    #[inline]
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Get the reference to the context.
    #[inline]
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Get the mutable reference to the context.
    #[inline]
    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

/// An identity operator that returns the input value.
pub struct Input<T>(PhantomData<fn() -> T>);

impl<T> fmt::Debug for Input<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Input").finish()
    }
}

impl<T> Clone for Input<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Input<T> {}

impl<T> Default for Input<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Operator<In<T>> for Input<T> {
    type Output = In<T>;

    #[inline]
    fn next(&mut self, input: In<T>) -> Self::Output {
        input
    }
}

/// Create an identity operator `Input` that returns the input value.
pub fn input<T>() -> Input<T> {
    Input(PhantomData)
}
