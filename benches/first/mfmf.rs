/*
* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_mfmf/seq/e20_light_Beg        time:   [108.07 ns 108.73 ns 109.43 ns]
first_mfmf/rayon/e20_light_Beg      time:   [1.6181 ms 1.6573 ms 1.6950 ms]
first_mfmf/orx/e20_light_Beg        time:   [1.1030 ms 1.1082 ms 1.1141 ms]

first_mfmf/seq/e20_light_Mid        time:   [245.06 µs 247.52 µs 250.03 µs]
first_mfmf/rayon/e20_light_Mid      time:   [11.598 ms 11.873 ms 12.087 ms]
first_mfmf/orx/e20_light_Mid        time:   [1.6074 ms 1.6190 ms 1.6311 ms]

first_mfmf/seq/e20_light_End        time:   [542.25 µs 548.52 µs 555.04 µs]
first_mfmf/rayon/e20_light_End      time:   [5.2982 ms 6.2473 ms 7.2232 ms]
first_mfmf/orx/e20_light_End        time:   [3.0353 ms 3.0902 ms 3.1476 ms]

first_mfmf/seq/e20_heavy_Beg        time:   [4.0804 µs 4.1668 µs 4.2704 µs]
first_mfmf/rayon/e20_heavy_Beg      time:   [2.1754 ms 2.2630 ms 2.3504 ms]
first_mfmf/orx/e20_heavy_Beg        time:   [1.5279 ms 1.5552 ms 1.5887 ms]

first_mfmf/seq/e20_heavy_Mid        time:   [12.893 ms 13.051 ms 13.219 ms]
first_mfmf/rayon/e20_heavy_Mid      time:   [7.1754 ms 7.5743 ms 7.9822 ms]
first_mfmf/orx/e20_heavy_Mid        time:   [3.7309 ms 3.7878 ms 3.8508 ms]

first_mfmf/seq/e20_heavy_End        time:   [23.796 ms 23.948 ms 24.103 ms]
first_mfmf/rayon/e20_heavy_End      time:   [7.1390 ms 7.3982 ms 7.6639 ms]
first_mfmf/orx/e20_heavy_End        time:   [5.2596 ms 5.3274 ms 5.3958 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::par;
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

fn h_m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    }
}

fn l_m(x: &u64) -> u64 {
    match *x {
        999 => 999,
        n => 7 * n + 1000,
    }
}

fn seq(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.iter();
    match h {
        false => iter
            .map(l_m)
            .filter(|x| *x == value)
            .map(|x| l_m(&x))
            .filter(|x| x.is_multiple_of(999))
            .next(),
        true => iter
            .map(h_m)
            .filter(|x| *x == value)
            .map(|x| h_m(&x))
            .filter(|x| x.is_multiple_of(999))
            .next(),
    }
}

fn orx(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = par(input.into_con_iter());
    match h {
        false => iter
            .map(l_m)
            .filter(|x| *x == value)
            .map(|x| l_m(&x))
            .filter(|x| x.is_multiple_of(999))
            .first(),
        true => iter
            .map(h_m)
            .filter(|x| *x == value)
            .map(|x| h_m(&x))
            .filter(|x| x.is_multiple_of(999))
            .first(),
    }
}

fn rayon(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.into_par_iter();
    match h {
        false => iter
            .map(l_m)
            .filter(|x| *x == value)
            .map(|x| l_m(&x))
            .filter(|x| x.is_multiple_of(999))
            .find_first(|_| true),
        true => iter
            .map(h_m)
            .filter(|x| *x == value)
            .map(|x| h_m(&x))
            .filter(|x| x.is_multiple_of(999))
            .find_first(|_| true),
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

    let mut group = c.benchmark_group("first_mfmf");

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
