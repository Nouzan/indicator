use super::QueueCapAtLeast;
use arrayvec::ArrayVec;

impl<T, const LEN: usize> QueueCapAtLeast<LEN> for ArrayVec<T, LEN> {
    type Item = T;

    fn empty() -> Self {
        Self::default()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn deque(&mut self) -> Option<Self::Item> {
        if !self.is_empty() {
            Some(self.remove(self.len() - 1))
        } else {
            None
        }
    }

    fn enque(&mut self, item: Self::Item) {
        self.insert(0, item);
    }

    fn get_latest(&self, n: usize) -> Option<&Self::Item> {
        self.get(n)
    }
}
