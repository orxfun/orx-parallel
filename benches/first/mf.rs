/*

* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_mf/seq/e20_light_Beg      time:   [231.82 ns 235.47 ns 239.49 ns]
first_mf/rayon/e20_light_Beg    time:   [3.7929 ms 4.2524 ms 4.8267 ms]
first_mf/orx/e20_light_Beg      time:   [1.9441 ms 1.9892 ms 2.0350 ms]

first_mf/seq/e20_light_Mid      time:   [445.99 µs 450.12 µs 454.30 µs]
first_mf/rayon/e20_light_Mid    time:   [17.842 ms 21.865 ms 27.117 ms]
first_mf/orx/e20_light_Mid      time:   [4.4227 ms 4.5705 ms 4.7208 ms]

first_mf/seq/e20_light_End      time:   [1.1407 ms 1.1608 ms 1.1817 ms]
first_mf/rayon/e20_light_End    time:   [21.873 ms 25.001 ms 29.443 ms]
first_mf/orx/e20_light_End      time:   [4.8992 ms 5.4526 ms 6.1000 ms]

first_mf/seq/e20_heavy_Beg      time:   [5.3141 µs 5.3808 µs 5.4498 µs]
first_mf/rayon/e20_heavy_Beg    time:   [4.2810 ms 4.5163 ms 4.7671 ms]
first_mf/orx/e20_heavy_Beg      time:   [1.9881 ms 2.1270 ms 2.2911 ms]

first_mf/seq/e20_heavy_Mid      time:   [18.854 ms 19.093 ms 19.336 ms]
first_mf/rayon/e20_heavy_Mid    time:   [14.989 ms 16.881 ms 19.086 ms]
first_mf/orx/e20_heavy_Mid      time:   [6.3391 ms 6.5511 ms 6.7769 ms]

first_mf/seq/e20_heavy_End      time:   [40.435 ms 41.204 ms 42.092 ms]
first_mf/rayon/e20_heavy_End    time:   [13.450 ms 14.076 ms 14.771 ms]
first_mf/orx/e20_heavy_End      time:   [7.0313 ms 8.1575 ms 9.7294 ms]
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
        false => iter.map(l_m).filter(|x| *x == value).next(),
        true => iter.map(h_m).filter(|x| *x == value).next(),
    }
}

fn orx(input: &[u64], h: bool, value: u64) -> Option<u64> {
    match h {
        false => input.into_par().map(l_m).filter(|x| *x == value).first(),
        true => input.into_par().map(h_m).filter(|x| *x == value).first(),
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
