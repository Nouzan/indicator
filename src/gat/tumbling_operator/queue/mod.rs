use core::ops::{Deref, Index};

/// Circular Queue.
pub mod circular;

/// Collection.
pub trait Collection {
    /// Create a new queue with the given capacity.
    fn with_capacity(cap: usize) -> Self;
}

/// Queue.
pub trait Queue {
    /// Item.
    type Item;

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

/// A change to the queue.
#[derive(Debug, Clone, Copy)]
pub enum Change<T> {
    /// Enqued.
    Push(Option<T>),
    /// The latest value has been updated.
    Swap(Option<T>),
}

impl<T> Change<T> {
    /// As ref.
    pub fn as_ref(&self) -> Change<&T> {
        match self {
            Self::Push(v) => Change::Push(v.as_ref()),
            Self::Swap(v) => Change::Swap(v.as_ref()),
        }
    }

    /// Convert into outdated.
    pub fn outdated(self) -> Option<T> {
        match self {
            Self::Push(v) => v,
            Self::Swap(v) => v,
        }
    }

    /// Check if it is a new peirod change (push).
    pub fn is_new_period(&self) -> bool {
        matches!(self, Self::Push(_))
    }

    fn as_push(&self) -> Option<&T> {
        if let Self::Push(v) = self {
            v.as_ref()
        } else {
            None
        }
    }

    fn as_swap(&self) -> Option<&T> {
        if let Self::Swap(v) = self {
            v.as_ref()
        } else {
            None
        }
    }
}

/// The core tumbling queue.
#[derive(Debug, Clone)]
pub struct Tumbling<Q: Queue>(Q, Change<Q::Item>);

impl<Q> Tumbling<Q>
where
    Q: Queue,
{
    pub(crate) fn new(queue: Q) -> Self {
        Self(queue, Change::Push(None))
    }

    /// Enque an item and deque the oldest item if overflow.
    fn enque_and_deque_overflow(&mut self, item: Q::Item) -> Option<Q::Item> {
        if self.0.is_full() {
            let oldest = self.0.deque();
            self.0.enque(item);
            oldest
        } else {
            self.0.enque(item);
            None
        }
    }

    /// Convert to a view of the queue.
    pub fn as_view<'a>(&'a self) -> View<'a, dyn Queue<Item = Q::Item> + 'a> {
        View {
            queue: &self.0,
            change: self.1.as_ref(),
        }
    }

    /// Convert to a [`QueueRef`].
    pub fn as_queue_ref(&self) -> QueueRef<'_, Q::Item> {
        QueueRef(self.as_view())
    }

    /// Push.
    #[inline]
    pub fn push(&mut self, item: Q::Item) -> Option<&Q::Item> {
        self.1 = Change::Push(self.enque_and_deque_overflow(item));
        self.1.as_push()
    }

    /// Swap.
    #[inline]
    pub fn swap(&mut self, mut item: Q::Item) -> Option<&Q::Item> {
        if let Some(head) = self.0.get_mut(0) {
            core::mem::swap(head, &mut item);
        }
        self.1 = Change::Swap(Some(item));
        self.1.as_swap()
    }

    /// Change.
    #[inline]
    pub fn change(&self) -> Change<&Q::Item> {
        self.1.as_ref()
    }
}

impl<Q> Deref for Tumbling<Q>
where
    Q: Queue,
{
    type Target = Q;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A view of the tumbling queue.
pub struct View<'a, Q: Queue + ?Sized> {
    queue: &'a Q,
    change: Change<&'a Q::Item>,
}

impl<'a, Q: Queue + ?Sized> Clone for View<'a, Q> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue,
            change: self.change,
        }
    }
}

impl<'a, Q: Queue + ?Sized> Copy for View<'a, Q> {}

impl<'a, Q: Queue + ?Sized> View<'a, Q> {
    /// Change.
    #[inline]
    pub fn change(&self) -> Change<&Q::Item> {
        self.change
    }
}

impl<'a, Q> Deref for View<'a, Q>
where
    Q: Queue + ?Sized,
{
    type Target = Q;

    fn deref(&self) -> &Self::Target {
        self.queue
    }
}

impl<'a, Q> Index<usize> for View<'a, Q>
where
    Q: Queue + ?Sized,
{
    type Output = Q::Item;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.queue.get(index).expect("index out of range")
    }
}

/// A reference of the tumbling queue.
pub struct QueueRef<'a, T>(View<'a, dyn Queue<Item = T> + 'a>);

impl<'a, T> Clone for QueueRef<'a, T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<'a, T> Copy for QueueRef<'a, T> {}

impl<'a, T> Deref for QueueRef<'a, T> {
    type Target = View<'a, dyn Queue<Item = T> + 'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl<Q> IndexMut<usize> for Tumbling<Q>
// where
//     Q: Queue,
// {
//     #[inline]
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         self.0.get_mut(index).expect("index out of range")
//     }
// }
