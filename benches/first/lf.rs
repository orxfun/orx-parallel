/*

* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_lf/seq/e20_light_Beg      time:   [1.2625 µs 1.2729 µs 1.2838 µs]
first_lf/rayon/e20_light_Beg    time:   [4.0181 ms 4.4393 ms 4.9073 ms]
first_lf/orx/e20_light_Beg      time:   [2.2935 ms 2.3699 ms 2.4485 ms]

first_lf/seq/e20_light_Mid      time:   [3.0526 ms 3.1017 ms 3.1592 ms]
first_lf/rayon/e20_light_Mid    time:   [15.849 ms 16.580 ms 17.370 ms]
first_lf/orx/e20_light_Mid      time:   [5.0026 ms 5.3335 ms 5.7252 ms]

first_lf/seq/e20_light_End      time:   [5.4910 ms 5.5630 ms 5.6358 ms]
first_lf/rayon/e20_light_End    time:   [27.774 ms 28.863 ms 29.966 ms]
first_lf/orx/e20_light_End      time:   [6.9977 ms 7.2638 ms 7.5496 ms]

first_lf/seq/e20_heavy_Beg      time:   [46.651 µs 47.222 µs 47.898 µs]
first_lf/rayon/e20_heavy_Beg    time:   [3.0950 ms 3.1916 ms 3.2945 ms]
first_lf/orx/e20_heavy_Beg      time:   [2.4598 ms 2.5285 ms 2.6012 ms]

first_lf/seq/e20_heavy_Mid      time:   [119.25 ms 122.21 ms 125.56 ms]
first_lf/rayon/e20_heavy_Mid    time:   [32.517 ms 34.125 ms 35.941 ms]
first_lf/orx/e20_heavy_Mid      time:   [17.983 ms 18.292 ms 18.632 ms]

first_lf/seq/e20_heavy_End      time:   [224.94 ms 228.13 ms 231.39 ms]
first_lf/rayon/e20_heavy_End    time:   [45.713 ms 46.771 ms 47.851 ms]
first_lf/orx/e20_heavy_End      time:   [35.471 ms 37.344 ms 39.499 ms]

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

fn h_l(x: &u64) -> impl IntoIterator<Item = u64> {
    (0..10).map(|i| match *x {
        999 => 999 + i,
        n => fibonacci(n % FIB_UPPER_BOUND) + 1000,
    })
}

fn l_l(x: &u64) -> impl IntoIterator<Item = u64> {
    (0..10).map(|i| match *x {
        999 => 999 + i,
        n => 7 * n + 1000,
    })
}

fn seq(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.iter();
    match h {
        false => iter.flat_map(l_l).filter(|x| *x == value).next(),
        true => iter.flat_map(h_l).filter(|x| *x == value).next(),
    }
}

fn orx(input: &[u64], h: bool, value: u64) -> Option<u64> {
    match h {
        false => input
            .into_par()
            .flat_map(l_l)
            .filter(|x| *x == value)
            .first(),
        true => input
            .into_par()
            .flat_map(h_l)
            .filter(|x| *x == value)
            .first(),
    }
}

fn rayon(input: &[u64], h: bool, value: u64) -> Option<u64> {
    let iter = input.into_par_iter();
    match h {
        false => iter
            .flat_map_iter(l_l)
            .filter(|x| *x == value)
            .find_first(|_| true),
        true => iter
            .flat_map_iter(h_l)
            .filter(|x| *x == value)
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
    heavy: bool,
    position: Pos,
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            position: Pos::Beg,
            val: 999,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            position: Pos::Mid,
            val: 999,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            position: Pos::End,
            val: 999,
            heavy: false,
        },
        Treat {
            len: 1 << 20,
            pos: 1 << 8,
            position: Pos::Beg,
            val: 999,
            heavy: true,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 19) + 7,
            position: Pos::Mid,
            val: 999,
            heavy: true,
        },
        Treat {
            len: 1 << 20,
            pos: (1 << 20) - 27,
            position: Pos::End,
            val: 999,
            heavy: true,
        },
    ];

    let mut group = c.benchmark_group("first_lf");

    for t in treatments {
        let name = format!(
            "e{}_{}_{:?}",
            t.len.ilog2(),
            match t.heavy {
                true => "heavy",
                false => "light",
            },
            t.position,
        );
        let input = inputs(t.len, t.pos, t.val);
        let expected = seq(&input, t.heavy, t.val);

        group.bench_with_input(BenchmarkId::new("seq", &name), &name, |b, _| {
            assert_eq!(&expected, &seq(&input, t.heavy, t.val));
            b.iter(|| seq(&input, t.heavy, t.val))
        });

        group.bench_with_input(BenchmarkId::new("rayon", &name), &name, |b, _| {
            assert_eq!(&expected, &rayon(&input, t.heavy, t.val));
            b.iter(|| rayon(&input, t.heavy, t.val))
        });

        group.bench_with_input(BenchmarkId::new("orx", &name), &name, |b, _| {
            assert_eq!(&expected, &orx(&input, t.heavy, t.val));
            b.iter(|| orx(&input, t.heavy, t.val))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
