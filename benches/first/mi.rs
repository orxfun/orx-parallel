/*

* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_mi/seq/e20_light_Beg      time:   [178.76 ns 180.16 ns 181.62 ns]
first_mi/rayon/e20_light_Beg    time:   [1.7342 ms 1.7676 ms 1.7994 ms]
first_mi/orx/e20_light_Beg      time:   [1.2874 ms 1.3069 ms 1.3282 ms]

first_mi/seq/e20_light_Mid      time:   [405.30 µs 409.87 µs 414.72 µs]
first_mi/rayon/e20_light_Mid    time:   [6.9570 ms 8.0187 ms 9.0736 ms]
first_mi/orx/e20_light_Mid      time:   [1.7888 ms 1.8207 ms 1.8581 ms]

first_mi/seq/e20_light_End      time:   [841.01 µs 853.36 µs 866.24 µs]
first_mi/rayon/e20_light_End    time:   [9.0571 ms 10.049 ms 11.036 ms]
first_mi/orx/e20_light_End      time:   [2.1431 ms 2.1779 ms 2.2143 ms]

first_mi/seq/e20_heavy_Beg      time:   [8.6038 µs 8.6946 µs 8.7987 µs]
first_mi/rayon/e20_heavy_Beg    time:   [1.7731 ms 1.9509 ms 2.1225 ms]
first_mi/orx/e20_heavy_Beg      time:   [1.4664 ms 1.4949 ms 1.5273 ms]

first_mi/seq/e20_heavy_Mid      time:   [24.281 ms 24.639 ms 25.038 ms]
first_mi/rayon/e20_heavy_Mid    time:   [4.1897 ms 4.5078 ms 4.8444 ms]
first_mi/orx/e20_heavy_Mid      time:   [4.9047 ms 4.9633 ms 5.0233 ms]

first_mi/seq/e20_heavy_End      time:   [48.971 ms 49.385 ms 49.839 ms]
first_mi/rayon/e20_heavy_End    time:   [7.8892 ms 8.3022 ms 8.7384 ms]
first_mi/orx/e20_heavy_End      time:   [7.5770 ms 7.6832 ms 7.7992 ms]

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

fn h_i(x: u64, value: u64) -> Option<u64> {
    let y = match x {
        999 => 999,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    };
    (y == value).then_some(2 * y + 7 + x)
}

fn l_i(x: u64, value: u64) -> Option<u64> {
    let y = match x {
        999 => 999,
        n => 7 * n + 1000,
    };
    (y == value).then_some(2 * y + 7 + x)
}

fn seq(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.iter();
    match h {
        false => iter.map(l_m).filter_map(|x| l_i(x, value)).next(),
        true => iter.map(h_m).filter_map(|x| h_i(x, value)).next(),
    }
}

fn orx(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = par(input.into_con_iter());
    match h {
        false => iter.map(l_m).filter_map(|x| l_i(x, value)).first(),
        true => iter.map(h_m).filter_map(|x| h_i(x, value)).first(),
    }
}

fn rayon(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.into_par_iter();
    match h {
        false => iter
            .map(l_m)
            .filter_map(|x| l_i(x, value))
            .find_first(|_| true),
        true => iter
            .map(h_m)
            .filter_map(|x| h_i(x, value))
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

    let mut group = c.benchmark_group("first_mi");

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
