use indicator::*;

fn boxed_example<I>(op1: Box<dyn Operator<I, Output = TickValue<usize>>>) -> impl Operator<I>
where
    I: Tickable<Value = usize> + Clone,
{
    let op2 = map_t(|x| x + 1);
    facet_t(op1, op2)
}

fn main() {
    let _op = boxed_example::<TickValue<usize>>(map_t(|x| x + 2).boxed());
}
