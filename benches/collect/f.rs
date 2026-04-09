/*

* eN means an input of size 2^N is used

reduce_f/seq/e15_light      time:   [16.344 µs 16.637 µs 16.960 µs]
reduce_f/rayon1/e15_light   time:   [8.9277 ms 9.2255 ms 9.5327 ms]
reduce_f/rayon2/e15_light   time:   [9.6452 ms 10.078 ms 10.513 ms]
reduce_f/orx/e15_light      time:   [1.5431 ms 1.5777 ms 1.6177 ms]

reduce_f/seq/e20_light      time:   [1.6849 ms 1.7096 ms 1.7364 ms]
reduce_f/rayon1/e20_light   time:   [14.893 ms 15.653 ms 16.369 ms]
reduce_f/rayon2/e20_light   time:   [17.352 ms 18.113 ms 18.831 ms]
reduce_f/orx/e20_light      time:   [2.2661 ms 2.2848 ms 2.3039 ms]

reduce_f/seq/e15_heavy      time:   [1.4048 ms 1.4240 ms 1.4458 ms]
reduce_f/rayon1/e15_heavy   time:   [10.024 ms 10.289 ms 10.541 ms]
reduce_f/rayon2/e15_heavy   time:   [9.5188 ms 10.053 ms 10.583 ms]
reduce_f/orx/e15_heavy      time:   [2.1792 ms 2.1978 ms 2.2179 ms]

reduce_f/seq/e20_heavy      time:   [46.712 ms 47.308 ms 47.936 ms]
reduce_f/rayon1/e20_heavy   time:   [13.850 ms 14.328 ms 14.816 ms]
reduce_f/rayon2/e20_heavy   time:   [12.358 ms 12.991 ms 13.655 ms]
reduce_f/orx/e20_heavy      time:   [8.4021 ms 8.5381 ms 8.6858 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f(a: &u64) -> bool {
    !(a + 7).is_multiple_of(11)
}

fn seq(input: &[u64]) -> Vec<u64> {
    input.iter().copied().filter(f).collect()
}

fn orx(input: &[u64]) -> Vec<u64> {
    input.into_par().copied().filter(f).collect()
}

fn rayon(input: &[u64]) -> Vec<u64> {
    input.into_par_iter().copied().filter(f).collect()
}

struct Treat {
    len: usize,
}

fn run(c: &mut Criterion) {
    let treatments = [Treat { len: 1 << 15 }, Treat { len: 1 << 20 }];

    let mut group = c.benchmark_group("col_f");

    for t in treatments {
        let name = format!("e{}", t.len.ilog2(),);
        let input = inputs(t.len);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input));
            b.iter(|| seq(&input))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon(&input));
            b.iter(|| rayon(&input))
        });

        group.bench_with_input(BenchmarkId::new("orx", &name), &name, |b, _| {
            assert_eq!(&expected, &orx(&input));
            b.iter(|| orx(&input))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
