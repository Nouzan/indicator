trait Operator<I> {
    type Output<'out>
    where
        Self: 'out,
        I: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out;
}

#[derive(Default)]
struct Collect {
    queue: Vec<usize>,
}

impl Operator<usize> for Collect {
    type Output<'out> = &'out [usize];

    fn next<'out>(&'out mut self, input: usize) -> Self::Output<'out>
    where
        usize: 'out,
    {
        self.queue.push(input);
        &self.queue
    }
}

struct Mux;

impl<'a> Operator<&'a [usize]> for Mux {
    type Output<'out> = Option<usize> where 'a: 'out;

    fn next<'out>(&'out mut self, input: &'a [usize]) -> Self::Output<'out>
    where
        'a: 'out,
    {
        std::thread::scope(|s| {
            let first = s.spawn(|| Some(input.first()? + 1));
            let second = s.spawn(|| Some(input.get(1)? + 1));
            let first = first.join().ok()??;
            let second = second.join().ok()??;
            Some(first + second)
        })
    }
}

pub struct Then<P1, P2>(P1, P2);

impl<I, P1, P2> Operator<I> for Then<P1, P2>
where
    P1: Operator<I>,
    P2: for<'out> Operator<P1::Output<'out>>,
{
    type Output<'out> = <P2 as Operator<<P1 as Operator<I>>::Output<'out>>>::Output<'out>
    where
        I: 'out,
        P1: 'out,
        P2: 'out;

    fn next<'out>(&'out mut self, input: I) -> Self::Output<'out>
    where
        I: 'out,
    {
        self.1.next(self.0.next(input))
    }
}

fn main() {
    let mut op = Then(Collect::default(), Mux);
    for x in [1, 2, 3, 4, 5] {
        println!("{:?}", op.next(x));
    }
}
