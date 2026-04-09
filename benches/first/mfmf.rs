/*
* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_mfmf/seq/e20_light_Beg    time:   [250.65 ns 252.57 ns 254.53 ns]
first_mfmf/rayon/e20_light_Beg  time:   [3.0536 ms 3.2758 ms 3.5572 ms]
first_mfmf/orx/e20_light_Beg    time:   [2.2740 ms 2.4014 ms 2.5433 ms]

first_mfmf/seq/e20_light_Mid    time:   [462.15 µs 471.12 µs 481.03 µs]
first_mfmf/rayon/e20_light_Mid  time:   [14.305 ms 15.129 ms 15.990 ms]
first_mfmf/orx/e20_light_Mid    time:   [2.7644 ms 2.8488 ms 2.9420 ms]

first_mfmf/seq/e20_light_End    time:   [984.30 µs 1.0179 ms 1.0623 ms]
first_mfmf/rayon/e20_light_End  time:   [16.129 ms 17.313 ms 18.574 ms]
first_mfmf/orx/e20_light_End    time:   [2.8953 ms 3.0124 ms 3.1485 ms]

first_mfmf/seq/e20_heavy_Beg    time:   [5.8278 µs 5.9379 µs 6.0362 µs]
first_mfmf/rayon/e20_heavy_Beg  time:   [3.9701 ms 4.1765 ms 4.3917 ms]
first_mfmf/orx/e20_heavy_Beg    time:   [3.4825 ms 3.9064 ms 4.3782 ms]

first_mfmf/seq/e20_heavy_Mid    time:   [19.943 ms 20.514 ms 21.183 ms]
first_mfmf/rayon/e20_heavy_Mid  time:   [21.469 ms 24.537 ms 28.335 ms]
first_mfmf/orx/e20_heavy_Mid    time:   [7.2474 ms 8.3068 ms 9.8094 ms]

first_mfmf/seq/e20_heavy_End    time:   [39.174 ms 39.907 ms 40.649 ms]
first_mfmf/rayon/e20_heavy_End  time:   [15.983 ms 17.149 ms 18.407 ms]
first_mfmf/orx/e20_heavy_End    time:   [7.3283 ms 7.6612 ms 8.0215 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
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
    match h {
        false => input
            .into_par()
            .map(l_m)
            .filter(|x| *x == value)
            .map(|x| l_m(&x))
            .filter(|x| x.is_multiple_of(999))
            .first(),
        true => input
            .into_par()
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
