use crate::{
    context::{ContextOperator, Value},
    Operator,
};

use super::Layer;

/// Optional layer.
/// It requires that the inner layer won't change the input and output of the input operator.
#[derive(Debug, Clone, Copy)]
pub struct OptionalLayer<L>(pub(crate) Option<L>);

impl<In, P, L> Layer<In, P> for OptionalLayer<L>
where
    P: ContextOperator<In>,
    L: Layer<In, P>,
    L::Operator: ContextOperator<In, Out = P::Out>,
{
    type Operator = Either<L::Operator, P>;

    type Out = P::Out;

    fn layer(&self, operator: P) -> Self::Operator {
        match &self.0 {
            Some(l) => Either::Left(l.layer(operator)),
            None => Either::Right(operator),
        }
    }
}

/// Either operator.
#[derive(Debug, Clone, Copy)]
pub enum Either<A, B> {
    /// Left.
    Left(A),
    /// Right.
    Right(B),
}

impl<In, Out, A, B> Operator<Value<In>> for Either<A, B>
where
    A: ContextOperator<In, Out = Out>,
    B: ContextOperator<In, Out = Out>,
{
    type Output = Value<Out>;

    fn next(&mut self, input: Value<In>) -> Self::Output {
        match self {
            Either::Left(a) => a.next(input),
            Either::Right(b) => b.next(input),
        }
    }
}
