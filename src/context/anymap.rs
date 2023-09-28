// Portions of this file are modifications based on work created and shared by [Original Author]
// and used according to terms described in the MIT License.
// Original Repository: https://github.com/hyperium/http
//
// The MIT License (MIT)
//
// Copyright (c) 2017 http-rs authors
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use alloc::{boxed::Box, fmt};
use core::{
    any::{Any, TypeId},
    hash::{BuildHasherDefault, Hasher},
};
use hashbrown::HashMap;

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<IdHasher>>;

// With TypeIds as keys, there's no need to hash them. They are already hashes
// themselves, coming from the compiler. The IdHasher just holds the u64 of
// the TypeId, and then returns it, instead of doing any bit fiddling.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// A type map to store values by type.
#[derive(Default)]
pub struct Context(Option<Box<AnyMap>>);

impl Context {
    /// Create an empty `Context`.
    #[inline]
    pub fn new() -> Self {
        Self(None)
    }

    /// Insert a type into the `Context`.
    pub fn insert<T: Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
        self.0
            .get_or_insert_with(Default::default)
            .insert(TypeId::of::<T>(), Box::new(value))
            .and_then(|boxed| boxed.downcast().ok().map(|v| *v))
    }

    /// Remove a type from the `Context`.
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.0
            .as_mut()
            .and_then(|map| map.remove(&TypeId::of::<T>()))
            .and_then(|boxed| boxed.downcast().ok().map(|v| *v))
    }

    /// Get a reference to a type from the `Context`.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.0
            .as_ref()
            .and_then(|map| map.get(&TypeId::of::<T>()))
            .and_then(|boxed| boxed.downcast_ref())
    }

    /// Get a mutable reference to a type from the `Context`.
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.0
            .as_mut()
            .and_then(|map| map.get_mut(&TypeId::of::<T>()))
            .and_then(|boxed| boxed.downcast_mut())
    }

    /// Clear the `Context`.
    #[inline]
    pub fn clear(&mut self) {
        if let Some(map) = self.0.as_mut() {
            map.clear();
        }
    }

    /// Check if the `Context` is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.as_ref().map_or(true, |map| map.is_empty())
    }

    /// Get the number of elements in the `Context`.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.as_ref().map_or(0, |map| map.len())
    }

    /// Extend the `Context` with the given `Context`.
    pub fn extend(&mut self, other: Context) {
        if let Some(other) = other.0 {
            if let Some(map) = self.0.as_mut() {
                map.extend(*other);
            } else {
                self.0 = Some(other);
            }
        }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context").field("len", &self.len()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_operations() {
        #[derive(Debug, PartialEq)]
        struct Foo(i32);

        let mut context = Context::new();
        assert!(context.is_empty());
        assert_eq!(context.len(), 0);

        context.insert(1);
        assert!(!context.is_empty());
        assert_eq!(context.len(), 1);

        context.insert(Foo(2));
        assert!(!context.is_empty());
        assert_eq!(context.len(), 2);

        let previous = context.insert(Foo(3));
        assert!(!context.is_empty());
        assert_eq!(context.len(), 2);
        assert_eq!(previous, Some(Foo(2)));

        assert_eq!(context.get::<i32>(), Some(&1));
        assert_eq!(context.get_mut::<Foo>(), Some(&mut Foo(3)));
        assert_eq!(context.remove::<Foo>(), Some(Foo(3)));
        assert_eq!(context.remove::<i32>(), Some(1));
        assert_eq!(context.remove::<i32>(), None);
        assert!(context.is_empty());
        assert_eq!(context.len(), 0);
    }
}
