use crate::{
    context::{ContextOperator, RefOperator, Value},
    Operator,
};

use super::Layer;

/// Layer that inserts a value into the context
/// with a [`RefOperator`].
pub struct Insert<F>(pub F);

impl<T, P, R, Out, F> Layer<T, P> for Insert<F>
where
    P: ContextOperator<T>,
    R: for<'a> RefOperator<'a, T, Output = Out>,
    Out: Send + Sync + 'static,
    F: Fn() -> R,
{
    type Output = InsertOperator<P, R>;

    #[inline]
    fn layer(&self, operator: P) -> Self::Output {
        InsertOperator {
            inner: operator,
            insert: (self.0)(),
        }
    }
}

/// Operator that inserts a value into the context.
pub struct InsertOperator<P, R> {
    inner: P,
    insert: R,
}

impl<T, P, R, Out> Operator<Value<T>> for InsertOperator<P, R>
where
    P: ContextOperator<T>,
    R: for<'a> RefOperator<'a, T, Output = Out>,
    Out: Send + Sync + 'static,
{
    type Output = P::Output;

    #[inline]
    fn next(&mut self, mut input: Value<T>) -> Self::Output {
        let value = self.insert.next(input.as_ref());
        input.context_mut().insert(value);
        self.inner.next(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::{layer::cache::Cache, output_with, ContextOperatorExt, ValueRef},
        IndicatorIteratorExt,
    };

    #[test]
    fn insert() {
        #[derive(Clone)]
        struct Foo;

        impl<'a> Operator<ValueRef<'a, f64>> for Foo {
            type Output = f64;

            fn next(&mut self, input: ValueRef<'a, f64>) -> Self::Output {
                *input.value * 2.0
            }
        }

        struct Bar;

        impl<'a> Operator<ValueRef<'a, f64>> for Bar {
            type Output = f64;

            fn next(&mut self, input: ValueRef<'a, f64>) -> Self::Output {
                let prev = input.context.get::<f64>().copied().unwrap_or(0.0);
                (*input.context.get::<f64>().unwrap() + 1.0 + prev) / 2.0
            }
        }

        let op = output_with(|| Bar)
            .with(Insert(|| Foo))
            .with(Cache::with_length(1.try_into().unwrap()))
            .finish();

        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(
            data.into_iter().indicator(op).collect::<Vec<_>>(),
            [2.5, 4.5, 6.5, 8.5, 10.5]
        );
    }
}
