#[cfg(feature = "array-vec")]
/// [`ArrayVec`] base queue.
pub mod arrayvec;

use crate::{Tick, TumblingWindow};

/// Queue that can hold at least `LEN` items.
pub trait QueueCapAtLeast<const LEN: usize> {
    /// Item type.
    type Item;

    /// Creat an empty queue.
    fn empty() -> Self;

    /// Get the nubmer of items in queue.
    fn len(&self) -> usize;

    /// Deque an item.
    fn deque(&mut self) -> Option<Self::Item>;

    /// Enque a new item.
    fn enque(&mut self, item: Self::Item);

    /// Get the `n`th latest item.
    ///
    /// The index is starting from `0`.
    fn get_latest(&self, n: usize) -> Option<&Self::Item>;

    /// Tumbling full.
    fn is_reach(&self) -> bool {
        self.len() >= LEN
    }

    /// Push a new item and return the oldest item if full.
    fn enque_and_deque_overflow(&mut self, item: Self::Item) -> Option<Self::Item> {
        if self.is_reach() {
            let oldest = self.deque();
            if LEN > 0 {
                self.enque(item);
            }
            oldest
        } else {
            if LEN > 0 {
                self.enque(item);
            }
            None
        }
    }
}

/// Queue used in [`TumblingOperation`](super::TumblingOperation).
#[derive(Debug, Clone)]
pub struct TumblingQueue<M: TumblingWindow, Q: QueueCapAtLeast<LEN>, const LEN: usize> {
    pub(super) mode: M,
    last_tick: Tick,
    pub(super) queue: Q,
}

impl<M: TumblingWindow, Q: QueueCapAtLeast<LEN>, const LEN: usize> TumblingQueue<M, Q, LEN> {
    /// Create a new tumbling queue from a mode.
    pub(crate) fn new(mode: M) -> Self {
        Self {
            mode,
            last_tick: Tick::BIG_BANG,
            queue: Q::empty(),
        }
    }

    /// Push or ignore.
    pub(crate) fn enque_or_ignore(
        &mut self,
        tick: &Tick,
        acc: &mut Option<Q::Item>,
    ) -> Option<Q::Item> {
        if !self.mode.same_window(&self.last_tick, tick) {
            self.last_tick = *tick;
            if let Some(item) = acc.take() {
                self.queue.enque_and_deque_overflow(item)
            } else {
                None
            }
        } else {
            None
        }
    }
}
