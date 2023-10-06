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
    type Operator = InsertOperator<P, R>;
    type Out = P::Out;

    #[inline]
    fn layer(&self, operator: P) -> Self::Operator {
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
    type Output = Value<P::Out>;

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
    R: for<'a> RefOperator<'a, T, Output = Option<Out>>,
    Out: Send + Sync + 'static,
    F: Fn() -> R,
{
    type Operator = InsertDataOperator<P, R>;
    type Out = P::Out;

    #[inline]
    fn layer(&self, operator: P) -> Self::Operator {
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
    R: for<'a> RefOperator<'a, T, Output = Option<Out>>,
    Out: Send + Sync + 'static,
{
    type Output = Value<P::Out>;

    #[inline]
    fn next(&mut self, input: Value<T>) -> Self::Output {
        let value = self.insert.next(input.as_ref());
        let mut out = self.inner.next(input);
        // We delay the insert after the inner operator is called.
        if let Some(value) = value {
            out.context_mut().data_mut().insert(value);
        }
        out
    }
}

/// Layer that inserts values into the env and data context at the same time.
/// with a [`RefOperator`].
pub struct InsertWithData<F>(pub F);

impl<T, P, R, Env, Data, F> Layer<T, P> for InsertWithData<F>
where
    P: ContextOperator<T>,
    F: Fn() -> R,
    R: for<'a> RefOperator<'a, T, Output = (Env, Option<Data>)>,
    Env: Send + Sync + 'static,
    Data: Send + Sync + 'static,
{
    type Operator = InsertWithDataOperator<P, R>;
    type Out = P::Out;

    #[inline]
    fn layer(&self, operator: P) -> Self::Operator {
        InsertWithDataOperator {
            inner: operator,
            insert: (self.0)(),
        }
    }
}

/// Operator that inserts a value into the data context.
pub struct InsertWithDataOperator<P, R> {
    inner: P,
    insert: R,
}

impl<T, P, R, Env, Data> Operator<Value<T>> for InsertWithDataOperator<P, R>
where
    P: ContextOperator<T>,
    R: for<'a> RefOperator<'a, T, Output = (Env, Option<Data>)>,
    Env: Send + Sync + 'static,
    Data: Send + Sync + 'static,
{
    type Output = Value<P::Out>;

    #[inline]
    fn next(&mut self, mut input: Value<T>) -> Self::Output {
        let (env, data) = self.insert.next(input.as_ref());
        // We insert the env immediately.
        input.context_mut().env_mut().insert(env);
        let mut out = self.inner.next(input);
        // But delay the insert of the data.
        if let Some(value) = data {
            out.context_mut().data_mut().insert(value);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        context::{
            insert_env_and_output, layer::cache::Cache, output, ContextOperatorExt, ValueRef,
        },
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

        let op = insert_env_and_output(|| Bar)
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

        impl<'a> Operator<ValueRef<'a, usize>> for Counter {
            type Output = Option<usize>;

            fn next(&mut self, input: ValueRef<'a, usize>) -> Self::Output {
                let mut should_insert = false;
                let count = input
                    .context
                    .data()
                    .get::<usize>()
                    .copied()
                    .unwrap_or_else(|| {
                        should_insert = true;
                        0
                    });
                if input.value % 2 == 0 {
                    Some(count + 1)
                } else if should_insert {
                    Some(count)
                } else {
                    None
                }
            }
        }

        let op = output(|_, ctx| ctx.data().get::<usize>().copied())
            .with(InsertData(|| Counter))
            .with(Cache::with_length(1.try_into().unwrap()))
            .finish();

        let data = [1, 2, 3, 4, 5];
        // Note that we only got the previous data because we inserted the data after the operator.
        assert_eq!(
            data.into_iter().indicator(op).collect::<Vec<_>>(),
            [None, Some(0), Some(1), Some(1), Some(2)]
        );
    }
}
