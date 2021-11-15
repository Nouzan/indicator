use crate::Operator;

/// Operated iterator.
#[derive(Debug, Clone, Copy)]
pub struct Operated<It, P> {
    source: It,
    op: P,
}

impl<It, P> Iterator for Operated<It, P>
where
    It: Iterator,
    P: Operator<It::Item>,
{
    type Item = P::Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.source.next().map(|input| self.op.next(input))
    }
}

/// Iterator extension trait for indicators.
pub trait IndicatorIteratorExt: Iterator {
    /// Apply an indicator to the iterator.
    fn indicator<P>(self, op: P) -> Operated<Self, P>
    where
        Self: Sized,
        P: Operator<Self::Item>,
    {
        Operated { source: self, op }
    }
}
