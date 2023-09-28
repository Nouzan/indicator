use core::ops::Deref;

use crate::{
    context::{Context, ContextOperator, IntoValue, Value},
    Operator,
};

use super::Layer;

/// Layer that *caches* and *clears* the final context,
/// and then provides it to the next evaluation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cache;

impl<T, P> Layer<T, P> for Cache
where
    P: ContextOperator<T>,
{
    type Output = CacheOperator<P>;

    fn layer(&self, inner: P) -> Self::Output {
        CacheOperator {
            inner,
            previous: Previous::default(),
        }
    }
}

/// Wrapper for previous context.
#[derive(Debug, Default)]
pub struct Previous(Context);

impl Previous {
    fn take(&mut self) -> Self {
        core::mem::take(self)
    }
}

impl Deref for Previous {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Operator for `Cache`.
#[derive(Debug)]
pub struct CacheOperator<P> {
    inner: P,
    previous: Previous,
}

impl<T, P> Operator<Value<T>> for CacheOperator<P>
where
    P: ContextOperator<T>,
{
    type Output = Value<<P::Output as IntoValue>::Inner>;

    fn next(&mut self, mut input: Value<T>) -> Self::Output {
        input.context_mut().insert(self.previous.take());
        let mut output = self.inner.next(input).into_value();
        self.previous
            .0
            .extend(core::mem::take(output.context_mut()));
        output
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::{input, layer::layer_fn},
        IndicatorIteratorExt,
    };

    use super::*;

    #[test]
    fn squre_cache() {
        struct Square<P>(P);

        impl<P> Operator<Value<i32>> for Square<P>
        where
            P: ContextOperator<i32>,
        {
            type Output = P::Output;

            fn next(&mut self, mut input: Value<i32>) -> Self::Output {
                input.apply(|v, ctx| {
                    let prev = ctx
                        .get::<Previous>()
                        .and_then(|prev| prev.get::<i32>().copied())
                        .unwrap_or(0);
                    println!("prev: {}", prev);
                    ctx.insert(prev.pow(2) + *v);
                });
                self.0.next(input)
            }
        }

        let op = input().with(layer_fn(|op| Square(op))).with(Cache).finish();

        let data = [1, 2, 3, 4, 5];
        data.into_iter().indicator(op).for_each(|v| {
            println!("input: {v}");
        });
    }
}
