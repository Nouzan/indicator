use indicator::gat::*;

#[derive(Default)]
struct Collect {
    queue: Vec<usize>,
}

impl GatOperator<usize> for Collect {
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

impl<'a> GatOperator<&'a [usize]> for Mux {
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

fn main() {
    let mut op = Collect::default().then(Mux);
    for x in [1, 2, 3, 4, 5] {
        println!("{:?}", op.next(x));
    }
}
