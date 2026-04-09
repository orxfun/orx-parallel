/*

* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_mi/seq/e20_light_Beg      time:   [317.27 ns 319.94 ns 322.67 ns]
first_mi/rayon/e20_light_Beg    time:   [2.8722 ms 2.9784 ms 3.0857 ms]
first_mi/orx/e20_light_Beg      time:   [1.8422 ms 1.8976 ms 1.9586 ms]

first_mi/seq/e20_light_Mid      time:   [640.63 µs 654.69 µs 671.80 µs]
first_mi/rayon/e20_light_Mid    time:   [14.004 ms 15.077 ms 16.226 ms]
first_mi/orx/e20_light_Mid      time:   [2.5569 ms 2.6079 ms 2.6605 ms]

first_mi/seq/e20_light_End      time:   [1.3228 ms 1.3405 ms 1.3575 ms]
first_mi/rayon/e20_light_End    time:   [11.463 ms 12.034 ms 12.614 ms]
first_mi/orx/e20_light_End      time:   [2.9951 ms 3.1095 ms 3.2314 ms]

first_mi/seq/e20_heavy_Beg      time:   [13.345 µs 13.473 µs 13.615 µs]
first_mi/rayon/e20_heavy_Beg    time:   [3.1574 ms 3.2652 ms 3.3770 ms]
first_mi/orx/e20_heavy_Beg      time:   [2.2114 ms 2.2876 ms 2.3719 ms]

first_mi/seq/e20_heavy_Mid      time:   [35.848 ms 36.455 ms 37.066 ms]
first_mi/rayon/e20_heavy_Mid    time:   [14.092 ms 14.748 ms 15.445 ms]
first_mi/orx/e20_heavy_Mid      time:   [7.3540 ms 8.2391 ms 9.5637 ms]

first_mi/seq/e20_heavy_End      time:   [71.276 ms 72.874 ms 74.532 ms]
first_mi/rayon/e20_heavy_End    time:   [21.677 ms 22.804 ms 23.995 ms]
first_mi/orx/e20_heavy_End      time:   [10.026 ms 10.209 ms 10.395 ms]

*/

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_iter::IntoConcurrentIter;
use orx_parallel::infallible::par;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

fn inputs(len: usize, pos: usize, val: u64) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    let mut vec = Vec::with_capacity(len);
    vec.extend((0..(len - 1)).map(|_| rng.random_range(0..150)));
    vec.insert(pos, val);
    vec
}

const FIB_UPPER_BOUND: u64 = 201;

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
