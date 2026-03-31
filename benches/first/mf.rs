/*

* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_mf/seq/e20_light_Beg      time:   [171.06 ns 173.19 ns 175.39 ns]
first_mf/rayon/e20_light_Beg    time:   [2.2812 ms 2.3469 ms 2.4142 ms]
first_mf/orx/e20_light_Beg      time:   [1.2468 ms 1.2645 ms 1.2865 ms]

first_mf/seq/e20_light_Mid      time:   [301.88 µs 305.15 µs 309.10 µs]
first_mf/rayon/e20_light_Mid    time:   [9.9078 ms 10.808 ms 11.729 ms]
first_mf/orx/e20_light_Mid      time:   [1.9223 ms 1.9422 ms 1.9634 ms]

first_mf/seq/e20_light_End      time:   [668.61 µs 677.45 µs 686.99 µs]
first_mf/rayon/e20_light_End    time:   [6.3642 ms 7.0027 ms 7.6539 ms]
first_mf/orx/e20_light_End      time:   [2.0640 ms 2.0950 ms 2.1280 ms]

first_mf/seq/e20_heavy_Beg      time:   [3.6742 µs 3.7150 µs 3.7565 µs]
first_mf/rayon/e20_heavy_Beg    time:   [2.7295 ms 2.8394 ms 2.9540 ms]
first_mf/orx/e20_heavy_Beg      time:   [1.3055 ms 1.3238 ms 1.3429 ms]

first_mf/seq/e20_heavy_Mid      time:   [14.159 ms 14.323 ms 14.493 ms]
first_mf/rayon/e20_heavy_Mid    time:   [7.0266 ms 7.6778 ms 8.3855 ms]
first_mf/orx/e20_heavy_Mid      time:   [4.4036 ms 4.5913 ms 4.7954 ms]

first_mf/seq/e20_heavy_End      time:   [27.862 ms 28.244 ms 28.635 ms]
first_mf/rayon/e20_heavy_End    time:   [7.8138 ms 8.2524 ms 8.7118 ms]
first_mf/orx/e20_heavy_End      time:   [6.2293 ms 6.3524 ms 6.4783 ms]
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
        false => iter.map(l_m).filter(|x| *x == value).next(),
        true => iter.map(h_m).filter(|x| *x == value).next(),
    }
}

fn orx(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = par(input.into_con_iter());
    match h {
        false => iter.map(l_m).filter(|x| *x == value).first(),
        true => iter.map(h_m).filter(|x| *x == value).first(),
    }
}

fn rayon(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.into_par_iter();
    match h {
        false => iter.map(l_m).filter(|x| *x == value).find_first(|_| true),
        true => iter.map(h_m).filter(|x| *x == value).find_first(|_| true),
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

    let mut group = c.benchmark_group("first_mf");

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
