/*

* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_lf/seq/e20_light_Beg      time:   [700.77 ns 707.53 ns 714.67 ns]
first_lf/orx/e20_light_Beg      time:   [1.1349 ms 1.1545 ms 1.1765 ms]

first_lf/seq/e20_light_Mid      time:   [1.4683 ms 1.4819 ms 1.4977 ms]
first_lf/orx/e20_light_Mid      time:   [1.9964 ms 2.0345 ms 2.0765 ms]

first_lf/seq/e20_light_End      time:   [3.3449 ms 3.3785 ms 3.4136 ms]
first_lf/orx/e20_light_End      time:   [2.7844 ms 2.8358 ms 2.8903 ms]

first_lf/seq/e20_heavy_Beg      time:   [26.922 µs 27.143 µs 27.393 µs]
first_lf/orx/e20_heavy_Beg      time:   [1.2524 ms 1.2692 ms 1.2898 ms]

first_lf/seq/e20_heavy_Mid      time:   [64.136 ms 64.587 ms 65.066 ms]
first_lf/orx/e20_heavy_Mid      time:   [8.9581 ms 9.0380 ms 9.1222 ms]

first_lf/seq/e20_heavy_End      time:   [126.39 ms 127.02 ms 127.68 ms]
first_lf/orx/e20_heavy_End      time:   [16.485 ms 16.676 ms 16.878 ms]

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
    let iter = par(input.into_con_iter());
    match h {
        false => iter.flat_map(l_l).filter(|x| *x == value).first(),
        true => iter.flat_map(h_l).filter(|x| *x == value).first(),
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

    let mut group = c.benchmark_group("first_lf");

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
