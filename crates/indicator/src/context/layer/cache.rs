use core::{num::NonZeroUsize, ops::Deref};

use crate::{
    context::{Context, ContextOperator, IntoValue, Value},
    Operator,
};

use super::Layer;

/// Layer that *caches* and *clears* the final context,
/// and then provides it to the next evaluation.
#[derive(Debug, Clone, Copy)]
pub struct Cache {
    length: NonZeroUsize,
}

impl Cache {
    /// Creates a new `Cache` layer with length set to `1`.
    pub fn new() -> Self {
        Self {
            length: NonZeroUsize::new(1).unwrap(),
        }
    }

    /// Creates a new `Cache` layer with the specified length.
    pub fn with_length(length: NonZeroUsize) -> Self {
        Self { length }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P> Layer<T, P> for Cache
where
    P: ContextOperator<T>,
{
    type Output = CacheOperator<P>;

    fn layer(&self, inner: P) -> Self::Output {
        CacheOperator {
            inner,
            previous: Previous::default(),
            limit: self.length.get() - 1,
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

    /// Iterates over the previous context.
    pub fn backward<F>(&self, mut f: F)
    where
        F: FnMut(&Context),
    {
        f(&self.0);
        if let Some(prev) = self.0.get::<Previous>() {
            prev.backward(f);
        }
    }

    /// Iterates over the previous context mutably.
    fn backward_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Context),
    {
        f(&mut self.0);
        if let Some(prev) = self.0.get_mut::<Previous>() {
            prev.backward_mut(f);
        }
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
    limit: usize,
}

impl<T, P> Operator<Value<T>> for CacheOperator<P>
where
    P: ContextOperator<T>,
{
    type Output = Value<<P::Output as IntoValue>::Inner>;

    fn next(&mut self, mut input: Value<T>) -> Self::Output {
        input.context_mut().insert(self.previous.take());
        let mut output = self.inner.next(input).into_value();
        // Remove the previous context if the limit is reached.
        // FIXME: There may be a better way to do this,
        // like using a `Vec` or slice to store the previous contexts
        // to avoid the recursive call.
        if self.limit > 0 {
            let limit = self.limit;
            let mut count = 1;
            output
                .context_mut()
                .get_mut::<Previous>()
                .unwrap()
                .backward_mut(|ctx| {
                    if count >= limit {
                        ctx.remove::<Previous>();
                    }
                    count += 1;
                });
        } else {
            output.context_mut().remove::<Previous>();
        }
        self.previous
            .0
            .extend(core::mem::take(output.context_mut()));
        output
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::{input, layer_fn, ContextOperatorExt},
        IndicatorIteratorExt, OperatorExt,
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
                    ctx.insert(prev.pow(2) + *v);
                });
                self.0.next(input)
            }
        }

        let op = input()
            .map(|input| {
                let previous = input.context().get::<Previous>().unwrap();
                let mut count = 0;
                previous.backward(|ctx| {
                    if let Some(v) = ctx.get::<i32>() {
                        println!("{count}: {v}");
                    }
                    count += 1;
                });
                input.map(|_, ctx| ctx.get::<i32>().copied().unwrap())
            })
            .with(layer_fn(|op| Square(op)))
            .with(Cache::with_length(2.try_into().unwrap()))
            .finish();

        let data = [1, 2, 3, 4, 5];
        data.into_iter().indicator(op).for_each(|v| {
            println!("current: {v}");
        });
    }
}
