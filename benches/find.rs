use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const TEST_LARGE_OUTPUT: bool = false;

const N: usize = 65_536 * 4;
const N_EARLY: usize = 1000;
const N_MIDDLE: usize = 65_536 * 2;
const N_LATE: usize = 65_536 * 4 - 10;
const N_NEVER: usize = usize::MAX;

const LARGE_OUTPUT_LEN: usize = match TEST_LARGE_OUTPUT {
    true => 64,
    false => 0,
};
const SEED: u64 = 9562;
const FIB_UPPER_BOUND: u32 = 201;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Output {
    id: String,
    name: String,
    numbers: [i64; LARGE_OUTPUT_LEN],
}

fn to_output(idx: &usize) -> Output {
    let idx = *idx;
    let prefix = match idx % 7 {
        0 => "zero-",
        3 => "three-",
        _ => "sth-",
    };
    let fib = fibonacci(&(idx as u32));
    let name = format!("{}-fib-{}", prefix, fib);

    let mut numbers = [0i64; LARGE_OUTPUT_LEN];
    for (i, x) in numbers.iter_mut().enumerate() {
        *x = match (idx * 7 + i) % 3 {
            0 => idx as i64 + i as i64,
            _ => idx as i64 - i as i64,
        };
    }

    let id = idx.to_string();

    Output { id, name, numbers }
}

fn get_find(n: usize) -> impl Fn(&Output) -> bool {
    move |a: &Output| a.id.parse::<usize>().unwrap() == n
}

fn fibonacci(n: &u32) -> u32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..*n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn inputs(len: usize) -> Vec<Output> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.gen_range(0..FIB_UPPER_BOUND) as usize)
        .map(|x| to_output(&x))
        .collect()
}

fn seq(inputs: &[Output], find: impl Fn(&Output) -> bool) -> Option<&Output> {
    inputs.into_iter().find(|x| find(x))
}

fn rayon(inputs: &[Output], find: impl Fn(&Output) -> bool + Send + Sync) -> Option<&Output> {
    use rayon::iter::ParallelIterator;
    inputs.into_par_iter().find_first(|x| find(x))
}

fn orx_sequential(
    inputs: &[Output],
    find: impl Fn(&Output) -> bool + Send + Sync,
) -> Option<&Output> {
    inputs.into_par().num_threads(1).find(|x| find(x))
}

fn orx(inputs: &[Output], find: impl Fn(&Output) -> bool + Send + Sync) -> Option<&Output> {
    inputs.into_par().find(|x| find(x))
}

fn run(c: &mut Criterion) {
    let treatments = [N_EARLY, N_MIDDLE, N_LATE, N_NEVER];

    let mut group = c.benchmark_group("find");

    for n_when in &treatments {
        let find = get_find(*n_when);
        let input = inputs(N);
        let expected = seq(&input, &find);

        let n_when = match *n_when {
            N_EARLY => "find-early",
            N_MIDDLE => "find-in-the-middle",
            N_LATE => "find-late",
            N_NEVER => "find-never",
            _ => panic!("unhandled n-when"),
        };

        group.bench_with_input(BenchmarkId::new("seq", n_when), n_when, |b, _| {
            assert_eq!(&expected, &seq(&input, &find));
            b.iter(|| seq(black_box(&input), &find))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n_when), n_when, |b, _| {
            assert_eq!(&expected, &rayon(&input, &find));
            b.iter(|| rayon(black_box(&input), &find))
        });

        group.bench_with_input(
            BenchmarkId::new("orx-sequential", n_when),
            n_when,
            |b, _| {
                assert_eq!(&expected, &orx_sequential(&input, &find));
                b.iter(|| orx_sequential(black_box(&input), &find))
            },
        );

        group.bench_with_input(BenchmarkId::new("orx", n_when), n_when, |b, _| {
            assert_eq!(&expected, &orx(&input, &find));
            b.iter(|| orx(black_box(&input), &find))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
