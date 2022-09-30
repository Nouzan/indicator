use super::TickQueueRef;

/// Help infer the right trait bound for closure.
pub fn queue_ref<I, O, F>(f: F) -> F
where
    F: for<'a> FnMut(TickQueueRef<'a, I>) -> O,
{
    f
}
