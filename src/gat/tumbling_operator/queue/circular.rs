//! # A circlar queue implemenation
//! ## The mind model
//! ```markdown
//! |------------------ grow ------------------>|
//! [0, .., head, .., tail, next_tail, .., cap-1]
//! ```
//! or
//! ```markdown
//! |------------------ grow ------------------>|
//! [0, .., tail, next_tail, .., head, .., cap-1]
//! ```

use tinyvec::TinyVec;

use super::{Collection, Queue};

/// Circular Queue backed by [`TinyVec`].
#[derive(Debug, Clone)]
pub struct Circular<T, const N: usize = 1> {
    inner: TinyVec<[Option<T>; N]>,
    cap: usize,
    next_tail: usize,
    head: Option<usize>,
}

impl<T, const N: usize> Circular<T, N> {
    fn entry_next_tail(&mut self) -> &mut Option<T> {
        // Assumption: `next_tail` must be a valid position, that is
        // 1) `0 <= next_tail < cap`,
        // 2) `inner[next_tail - 1]` is `Some` or `next_tail == 0`.
        if self.inner.get(self.next_tail).is_none() {
            // |---- grow ---->|
            // |2|1|0|next_tail|
            self.inner.push(None);
        }
        self.inner.get_mut(self.next_tail).unwrap()
    }

    fn move_next_tail(&mut self) {
        if self.head.is_none() {
            self.head = Some(self.next_tail);
        }
        self.next_tail = (self.next_tail + 1) % self.cap;
    }

    fn move_head(&mut self) {
        let head = (self.head.unwrap() + 1) % self.cap;
        if head == self.next_tail {
            self.head = None;
        } else {
            self.head = Some(head);
        }
    }

    fn to_idx(&self, idx: usize) -> Option<usize> {
        let offset = idx + 1;
        if self.next_tail >= offset {
            Some(self.next_tail - offset)
        } else if self.next_tail + self.cap >= offset {
            Some(self.next_tail + self.cap - offset)
        } else {
            None
        }
    }
}

impl<T, const N: usize> Collection for Circular<T, N> {
    fn with_capacity(cap: usize) -> Self {
        assert!(cap != 0, "capacity cannot be zero");
        Self {
            inner: TinyVec::with_capacity(cap),
            cap,
            next_tail: 0,
            head: None,
        }
    }
}

impl<T, const N: usize> Queue for Circular<T, N> {
    type Item = T;

    fn enque(&mut self, item: Self::Item) {
        let next_tail = self.entry_next_tail();
        assert!(next_tail.is_none(), "queue is full");
        *next_tail = Some(item);
        self.move_next_tail();
    }

    fn deque(&mut self) -> Option<Self::Item> {
        let head = self.head?;
        let item = self.inner.get_mut(head).unwrap().take();
        self.move_head();
        item
    }

    fn len(&self) -> usize {
        if let Some(head) = self.head {
            if self.next_tail > head {
                self.next_tail - head
            } else {
                self.cap - (head - self.next_tail)
            }
        } else {
            0
        }
    }

    fn cap(&self) -> usize {
        self.cap
    }

    fn get(&self, idx: usize) -> Option<&Self::Item> {
        let idx = self.to_idx(idx)?;
        self.inner.get(idx)?.as_ref()
    }

    fn get_mut(&mut self, idx: usize) -> Option<&mut Self::Item> {
        let idx = self.to_idx(idx)?;
        self.inner.get_mut(idx)?.as_mut()
    }

    #[inline]
    fn is_inline(&self) -> bool {
        self.inner.is_inline()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut queue = Circular::<_, 3>::with_capacity(3);
        assert!(queue.inner.is_inline());
        assert!(queue.is_empty());
        queue.enque(1);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.get(0), Some(&1));
        assert!(queue.get(1).is_none());
        assert_eq!(queue.deque(), Some(1));
        assert_eq!(queue.len(), 0);
        assert!(queue.get(0).is_none());
    }

    #[test]
    fn full() {
        let mut queue = Circular::<_, 1>::with_capacity(3);
        assert!(queue.inner.is_heap());
        queue.enque(1);
        queue.enque(2);
        queue.enque(3);
        assert!(queue.is_full());
        assert_eq!(queue.deque(), Some(1));
        assert_eq!(queue.deque(), Some(2));
        assert_eq!(queue.deque(), Some(3));
        assert_eq!(queue.deque(), None);
        assert!(queue.is_empty());
    }

    #[test]
    fn circular_1() {
        let mut queue = Circular::<_, 1>::with_capacity(3);
        queue.enque(1);
        assert_eq!(queue.deque(), Some(1));
        queue.enque(2);
        queue.enque(3);
        queue.enque(4);
        assert!(queue.is_full());
        assert_eq!(queue.deque(), Some(2));
        assert_eq!(queue.get(0), Some(&4));
        assert_eq!(queue.get(1), Some(&3));
        assert_eq!(queue.get(2), None);
        assert_eq!(queue.deque(), Some(3));
        assert_eq!(queue.deque(), Some(4));
        assert!(queue.is_empty());
        assert_eq!(queue.deque(), None);
    }

    #[test]
    fn circular_2() {
        let mut queue = Circular::<_, 1>::with_capacity(3);
        queue.enque(1);
        assert_eq!(queue.deque(), Some(1));
        queue.enque(2);
        assert_eq!(queue.deque(), Some(2));
        queue.enque(3);
        assert_eq!(queue.deque(), Some(3));
        queue.enque(4);
        assert_eq!(queue.deque(), Some(4));
        queue.enque(5);
        assert_eq!(queue.deque(), Some(5));
        queue.enque(6);
        assert_eq!(queue.deque(), Some(6));
        queue.enque(7);
        assert_eq!(queue.deque(), Some(7));
    }
}
