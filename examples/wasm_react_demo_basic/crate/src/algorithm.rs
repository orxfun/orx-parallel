#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
use orx_parallel::{IntoParIter, Par};

pub const MAX_N: u32 = 93;

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
pub fn fib_sum_parallel(start: u32, end: u32) -> u64 {
    let range_start = start as usize;
    let range_end_exclusive = (end as usize) + 1;

    (range_start..range_end_exclusive)
        .into_par()
        .map(|n| fibonacci(n as u32))
        .sum()
}

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let next = a.saturating_add(b);
                a = b;
                b = next;
            }
            b
        }
    }
}
