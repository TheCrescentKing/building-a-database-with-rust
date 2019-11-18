use std::time::Instant;

fn main() {
    let time0 = Instant::now();
    for i in 0..10_000 {
        println!("{}", i);
    }
    println!("{:?}", time0.elapsed());
}
