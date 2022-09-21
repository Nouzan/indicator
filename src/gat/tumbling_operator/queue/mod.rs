use core::ops::Deref;

/// Circular Queue.
pub mod circular;

/// Queue.
pub trait Queue {
    /// Item.
    type Item;

    /// Create a new queue with the given capacity.
    fn with_capacity(cap: usize) -> Self;

    /// Enque.
    fn enque(&mut self, item: Self::Item);

    /// Deque.
    fn deque(&mut self) -> Option<Self::Item>;

    /// Length.
    fn len(&self) -> usize;

    /// Capacity.
    fn cap(&self) -> usize;

    /// Get a reference of the item in the given position (tail position is `0`).
    fn get(&self, idx: usize) -> Option<&Self::Item>;

    /// Get a mutable reference of the item in the given position (tail position is `0`).
    fn get_mut(&mut self, idx: usize) -> Option<&mut Self::Item>;

    /// Is empty.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Is full.
    #[inline]
    fn is_full(&self) -> bool {
        self.len() == self.cap()
    }

    /// Returns whether elements are on stack.
    fn is_inline(&self) -> bool;
}

/// The core tumbling queue.
#[derive(Debug, Clone)]
pub struct TumblingQueue<T>(T);

impl<T> TumblingQueue<T>
where
    T: Queue,
{
    /// Enque an item and deque the oldest item if overflow.
    pub fn enque_and_deque_overflow(&mut self, item: T::Item) -> Option<T::Item> {
        if self.0.is_full() {
            let oldest = self.0.deque();
            self.0.enque(item);
            oldest
        } else {
            self.0.enque(item);
            None
        }
    }
}

impl<T> Deref for TumblingQueue<T>
where
    T: Queue,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
