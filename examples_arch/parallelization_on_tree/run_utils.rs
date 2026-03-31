use crate::amount_of_work;
use std::time::Instant;

pub fn timed<F, L, T>(name: &'static str, fun: F, log: L)
where
    F: Fn() -> T,
    L: Fn(T),
{
    println!("> {name}");
    let start = Instant::now();

    let result = fun();

    let elapsed = start.elapsed();

    println!("  elapsed = {elapsed:?}");
    log(result);
    println!();
}

/// Fibonacci as example computation on each of the node values.
pub fn compute(value: u64) -> u64 {
    (0..*amount_of_work())
        .map(|j| {
            let n = core::hint::black_box(value + j as u64);
            let mut a = 0;
            let mut b = 1;
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            a
        })
        .sum()
}
