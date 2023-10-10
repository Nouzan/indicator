use indicator::prelude::*;
use num::Num;
use wasm_bindgen::prelude::*;

#[operator(input = I, generate_out_with_data)]
fn ma<T>(
    #[input] x: &T,
    #[data(as_ref?)] prev: Option<&T>,
    #[data(as_ref)] alpha: &T,
) -> (T, Option<T>)
where
    T: Num + Clone,
{
    let prev = prev.cloned().unwrap_or_else(|| x.clone());
    let out = x.clone() * alpha.clone() + prev.clone() * (T::one() - alpha.clone());
    (out.clone(), Some(out))
}

/// Wasm Ma Operator.
#[wasm_bindgen]
pub struct Ma(BoxOperator<'static, f64, f64>);

#[wasm_bindgen]
impl Ma {
    pub fn new(alpha: f64) -> Self {
        use derive_more::{AsRef, From};

        #[derive(AsRef, Clone)]
        struct Alpha(f64);

        #[derive(AsRef, From)]
        struct State(f64);

        let op = insert_and_output(ma::<f64, _, State, Alpha, f64, State>)
            .provide(Alpha(alpha))
            .finish()
            .boxed();

        Ma(op)
    }

    pub fn next(&mut self, x: f64) -> f64 {
        self.0.next(x)
    }
}
