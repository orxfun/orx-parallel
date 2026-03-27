/*

* light & heavy show the intensity of computation
* beg & mid & end show where the element to be found is located

first_i/seq/e20_light_Beg      time:   [142.46 ns 143.53 ns 144.67 ns]
first_i/rayon/e20_light_Beg    time:   [2.2991 ms 2.4693 ms 2.6570 ms]
first_i/orx/e20_light_Beg      time:   [1.3613 ms 1.3732 ms 1.3849 ms]

first_i/seq/e20_light_Mid      time:   [295.70 µs 297.98 µs 300.31 µs]
first_i/rayon/e20_light_Mid    time:   [9.4923 ms 10.145 ms 10.779 ms]
first_i/orx/e20_light_Mid      time:   [1.8228 ms 1.8494 ms 1.8777 ms]

first_i/seq/e20_light_End      time:   [589.91 µs 596.68 µs 604.39 µs]
first_i/rayon/e20_light_End    time:   [4.7884 ms 5.6787 ms 6.6069 ms]
first_i/orx/e20_light_End      time:   [1.9493 ms 1.9717 ms 1.9945 ms]

first_i/seq/e20_heavy_Beg      time:   [3.3971 µs 3.4974 µs 3.5955 µs]
first_i/rayon/e20_heavy_Beg    time:   [2.3477 ms 2.4409 ms 2.5415 ms]
first_i/orx/e20_heavy_Beg      time:   [1.3980 ms 1.4086 ms 1.4210 ms]

first_i/seq/e20_heavy_Mid      time:   [11.806 ms 11.888 ms 11.968 ms]
first_i/rayon/e20_heavy_Mid    time:   [5.2351 ms 5.6002 ms 5.9659 ms]
first_i/orx/e20_heavy_Mid      time:   [3.6942 ms 3.7424 ms 3.7953 ms]

first_i/seq/e20_heavy_End      time:   [24.981 ms 25.182 ms 25.391 ms]
first_i/rayon/e20_heavy_End    time:   [5.4355 ms 5.9478 ms 6.4877 ms]
first_i/orx/e20_heavy_End      time:   [5.2835 ms 5.3641 ms 5.4515 ms]

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
