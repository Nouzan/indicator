use indicator::*;

fn main() {
    let mut m = map(|x| x + 1);
    let mut op = (&mut m).then(map(|x| x + 2));
    for x in [1, 2, 3, 4, 5] {
        println!("{:?}", op.next(x));
    }
}
