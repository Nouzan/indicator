use super::QueueRef;

/// Help infer the right trait bound for closure.
pub fn queue_ref<I, O, F>(f: F) -> F
where
    F: for<'a> FnMut(QueueRef<'a, I>) -> O,
{
    f
}
