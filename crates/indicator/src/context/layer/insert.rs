use crate::{
    context::{ContextOperator, RefOperator, Value},
    Operator,
};

use super::Layer;

/// Layer that inserts a value into the env context
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

/// Operator that inserts a value into the env context.
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
        input.context_mut().env_mut().insert(value);
        self.inner.next(input)
    }
}

/// Layer that inserts a value into the data context
/// with a [`RefOperator`].
pub struct InsertData<F>(pub F);

impl<T, P, R, Out, F> Layer<T, P> for InsertData<F>
where
    P: ContextOperator<T>,
    R: for<'a> RefOperator<'a, T, Output = Out>,
    Out: Send + Sync + 'static,
    F: Fn() -> R,
{
    type Output = InsertDataOperator<P, R>;

    #[inline]
    fn layer(&self, operator: P) -> Self::Output {
        InsertDataOperator {
            inner: operator,
            insert: (self.0)(),
        }
    }
}

/// Operator that inserts a value into the data context.
pub struct InsertDataOperator<P, R> {
    inner: P,
    insert: R,
}

impl<T, P, R, Out> Operator<Value<T>> for InsertDataOperator<P, R>
where
    P: ContextOperator<T>,
    R: for<'a> RefOperator<'a, T, Output = Out>,
    Out: Send + Sync + 'static,
{
    type Output = P::Output;

    #[inline]
    fn next(&mut self, mut input: Value<T>) -> Self::Output {
        let value = self.insert.next(input.as_ref());
        input.context_mut().data_mut().insert(value);
        self.inner.next(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::{layer::cache::Cache, output, output_with, ContextOperatorExt, ValueRef},
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
                // Read from the previous env value.
                let prev = input.context.env().get::<f64>().copied().unwrap_or(0.0);
                (*input.context.env().get::<f64>().unwrap() + 1.0 + prev) / 2.0
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

    #[test]
    fn insert_data() {
        #[derive(Clone)]
        struct Counter;

        impl<'a> Operator<ValueRef<'a, f64>> for Counter {
            type Output = usize;

            fn next(&mut self, input: ValueRef<'a, f64>) -> Self::Output {
                let count = input.context.data().get::<usize>().copied().unwrap_or(0);
                count + 1
            }
        }

        let op = output(|_, ctx| {
            let count = ctx.data().get::<usize>().copied().unwrap();
            count as f64
        })
        .with(InsertData(|| Counter))
        .with(Cache::with_length(1.try_into().unwrap()))
        .finish();

        let data = [1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(
            data.into_iter().indicator(op).collect::<Vec<_>>(),
            [1.0, 2.0, 3.0, 4.0, 5.0]
        );
    }
}
