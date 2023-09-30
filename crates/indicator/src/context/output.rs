use crate::Operator;

use super::{Context, RefOperator, Value};

/// An operator for outputting.
pub struct Output<F>(F);

impl<I, O, F> Operator<Value<I>> for Output<F>
where
    F: FnMut(I, &mut Context) -> O,
{
    type Output = Value<O>;

    fn next(&mut self, input: Value<I>) -> Self::Output {
        input.map(&mut self.0)
    }
}

/// Create an operator for outputting from a closure.
pub fn output<I, O>(
    f: impl FnMut(I, &mut Context) -> O,
) -> Output<impl FnMut(I, &mut Context) -> O> {
    Output(f)
}

/// An output operator that insert the output into the context.
pub struct OutputWith<R>(R);

impl<T, R, Out> Operator<Value<T>> for OutputWith<R>
where
    R: for<'a> RefOperator<'a, T, Output = Out>,
    Out: Clone + Send + Sync + 'static,
{
    type Output = Value<Out>;

    #[inline]
    fn next(&mut self, mut input: Value<T>) -> Self::Output {
        let value = self.0.next(input.as_ref());
        input.context_mut().env_mut().insert(value.clone());
        input.map(|_, _| value)
    }
}

/// Create an output operator that insert the output into the context.
pub fn output_with<I, O, R, F>(operator: F) -> OutputWith<R>
where
    R: for<'a> RefOperator<'a, I, Output = O>,
    O: Clone + Send + Sync + 'static,
    F: FnOnce() -> R,
{
    OutputWith(operator())
}
