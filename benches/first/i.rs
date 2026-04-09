/*

* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_i/seq/e20_light_Beg   time:   [239.71 ns 242.16 ns 244.57 ns]
first_i/rayon/e20_light_Beg time:   [2.6938 ms 2.7724 ms 2.8532 ms]
first_i/orx/e20_light_Beg   time:   [2.5453 ms 2.6665 ms 2.7921 ms]

first_i/seq/e20_light_Mid   time:   [521.98 µs 538.00 µs 554.69 µs]
first_i/rayon/e20_light_Mid time:   [23.360 ms 26.073 ms 29.243 ms]
first_i/orx/e20_light_Mid   time:   [3.0294 ms 3.1316 ms 3.2448 ms]

first_i/seq/e20_light_End   time:   [983.36 µs 1.0010 ms 1.0199 ms]
first_i/rayon/e20_light_End time:   [19.914 ms 21.465 ms 23.105 ms]
first_i/orx/e20_light_End   time:   [3.2294 ms 3.3432 ms 3.4605 ms]

first_i/seq/e20_heavy_Beg   time:   [5.8557 µs 5.9888 µs 6.1635 µs]
first_i/rayon/e20_heavy_Beg time:   [3.4674 ms 3.7918 ms 4.2091 ms]
first_i/orx/e20_heavy_Beg   time:   [2.2543 ms 2.3611 ms 2.4746 ms]

first_i/seq/e20_heavy_Mid   time:   [20.076 ms 20.460 ms 20.887 ms]
first_i/rayon/e20_heavy_Mid time:   [17.093 ms 19.354 ms 22.671 ms]
first_i/orx/e20_heavy_Mid   time:   [6.7206 ms 6.9126 ms 7.1136 ms]

first_i/seq/e20_heavy_End   time:   [39.386 ms 39.909 ms 40.444 ms]
first_i/rayon/e20_heavy_End time:   [16.748 ms 17.493 ms 18.257 ms]
first_i/orx/e20_heavy_End   time:   [8.3474 ms 8.6485 ms 8.9615 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::infallible::par;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const FIB_UPPER_BOUND: u64 = 201;

fn inputs(len: usize, pos: usize, val: u64) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut vec = Vec::with_capacity(len);
    vec.extend((0..(len - 1)).map(|_| rng.random_range(0..150)));
    vec.insert(pos, val);
    vec
}

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn h_i(x: &u64, value: u64) -> Option<u64> {
    let y = match *x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    };
    (y == value).then_some(2 * y + 7 + x)
}

fn l_i(x: &u64, value: u64) -> Option<u64> {
    let y = match *x {
        999 => 999,
        n => 7 * n + 1000,
    };
    (y == value).then_some(2 * y + 7 + x)
}

fn seq(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.iter();
    match h {
        false => iter.filter_map(|x| l_i(x, value)).next(),
        true => iter.filter_map(|x| h_i(x, value)).next(),
    }
}

fn orx(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = par(input.into_con_iter());
    match h {
        false => iter.filter_map(|x| l_i(x, value)).first(),
        true => iter.filter_map(|x| h_i(x, value)).first(),
    }
}

fn rayon(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.into_par_iter();
    match h {
        false => iter.filter_map(|x| l_i(x, value)).find_first(|_| true),
        true => iter.filter_map(|x| h_i(x, value)).find_first(|_| true),
    }
}

#[derive(Debug)]
enum Pos {
    Beg,
    Mid,
    End,
}

struct Treat {
    len: usize,
    pos: usize,
    val: u64,
    heavy_compute: bool,
    position: Pos,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            position: Pos::Beg,
            val: 999,
            heavy_compute: false,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            position: Pos::Mid,
            val: 999,
            heavy_compute: false,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            position: Pos::End,
            val: 999,
            heavy_compute: false,
        },
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            position: Pos::Beg,
            val: 999,
            heavy_compute: true,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            position: Pos::Mid,
            val: 999,
            heavy_compute: true,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            position: Pos::End,
            val: 999,
            heavy_compute: true,
        },
    ];

    let mut group = c.benchmark_group("first_i");

    for t in treatments {
        let name = format!(
            "e{}_{}_{:?}",
            t.len.ilog2(),
            match t.heavy_compute {
                true => "heavy",
                false => "light",
            },
            t.position,
        );
        let input = inputs(t.len, t.pos, t.val);
        let expected = seq(&input, t.heavy_compute, t.val);

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.heavy_compute, t.val));
            b.iter(|| seq(&input, t.heavy_compute, t.val))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.heavy_compute, t.val));
            b.iter(|| rayon(&input, t.heavy_compute, t.val))
        });

        group.bench_with_input(BenchmarkId::new("orx", &name), &name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.heavy_compute, t.val));
            b.iter(|| orx(&input, t.heavy_compute, t.val))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
