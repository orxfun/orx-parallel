pub fn test_different_params<T>(test: T)
where
    T: Fn(usize, usize),
{
    let params = [[1, 1], [2, 2], [4, 1024], [16, 64], [8, 32]];

    for [num_threads, chunk_size] in params {
        test(num_threads, chunk_size);
    }
}

pub fn test_reduce<T>(test: T)
where
    T: Fn(usize, usize, usize),
{
    let params = [
        [0, 1, 1],
        [0, 2, 2],
        [1, 3, 1],
        [64, 4, 1024],
        [1024, 1, 1],
        [4999, 4, 64],
        [7850, 8, 8],
    ];

    for [len, num_threads, chunk_size] in params {
        test(len, num_threads, chunk_size);
    }
}

use rayon::prelude::*;
use std::hint::black_box;

fn abc() {
    let t0 = std::time::Instant::now();
    let result = (0..512)
        .map(|x| {
            (0..512)
                .map(|y| {
                    (0..16)
                        .into_par_iter()
                        .map(|z| black_box(x * y * z))
                        .sum::<usize>()
                })
                .sum::<usize>()
        })
        .sum::<usize>();
    println!("elapsed: {} ms", t0.elapsed().as_millis());
    println!("result: {result}");
}
