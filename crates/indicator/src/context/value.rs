use core::{fmt, marker::PhantomData};

use crate::Operator;

use super::{anymap::Map, Context};

/// Value with context.
#[derive(Debug)]
pub struct Value<T> {
    /// Inner value.
    pub(crate) value: T,
    /// Context.
    pub(crate) context: Context,
}

impl<T> Value<T> {
    /// Create a new `Value` with the given value.
    #[inline]
    pub(super) fn with_data(value: T, data: Map) -> Self {
        Self {
            value,
            context: Context::with_data(data),
        }
    }

    /// Get the reference to the inner value.
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Get the mutable reference to the inner value.
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

    /// Apply a closure to the inner value and the context.
    pub fn apply(&mut self, f: impl FnOnce(&mut T, &mut Context)) {
        f(&mut self.value, &mut self.context)
    }

    /// Convert the inner value.
    pub fn map<U>(self, f: impl FnOnce(T, &mut Context) -> U) -> Value<U> {
        let Self { value, mut context } = self;
        Value {
            value: f(value, &mut context),
            context,
        }
    }

    /// Convert into the inner value.
    pub(super) fn into_inner(self) -> T {
        self.value
    }

    /// Create a reference to the value.
    pub fn as_ref(&self) -> ValueRef<'_, T> {
        ValueRef {
            value: &self.value,
            context: &self.context,
        }
    }
}

/// An identity operator that just returns the input.
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

impl<T> Operator<Value<T>> for Input<T> {
    type Output = Value<T>;

    #[inline]
    fn next(&mut self, input: Value<T>) -> Self::Output {
        input
    }
}

/// Create an identity operator `Input` that returns the input value.
pub fn input<T>() -> Input<T> {
    Input(PhantomData)
}

/// Reference to a [`Value`].
#[derive(Debug)]
pub struct ValueRef<'a, T> {
    /// Reference to the inner value.
    pub(crate) value: &'a T,
    /// Reference to the context.
    pub(crate) context: &'a Context,
}

impl<'a, T> ValueRef<'a, T> {
    /// Get the reference to the inner value.
    #[inline]
    pub fn value(&self) -> &'a T {
        self.value
    }

    /// Get the reference to the context.
    #[inline]
    pub fn context(&self) -> &'a Context {
        self.context
    }
}
