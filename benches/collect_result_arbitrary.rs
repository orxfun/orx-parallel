use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::num::ParseIntError;

type ERR = ParseIntError;

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
struct Input {
    id: String,
    name: String,
    numbers: [i64; LARGE_OUTPUT_LEN],
}

fn to_input(idx: &usize) -> Input {
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

    Input { id, name, numbers }
}

fn to_bad_input() -> Input {
    Input {
        id: "xyz".to_string(),
        name: "xyz".to_string(),
        numbers: Default::default(),
    }
}

fn map_input_to_result(input: &Input) -> Result<String, ERR> {
    match input.id.parse::<usize>() {
        Ok(_) => Ok(input.id.clone()),
        Err(e) => Err(e),
    }
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

fn inputs(len: usize, idx_error: usize) -> Vec<Input> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|i| match i == idx_error {
            true => to_bad_input(),
            false => {
                let x = rng.random_range(0..FIB_UPPER_BOUND) as usize;
                to_input(&x)
            }
        })
        .collect()
}

fn seq(inputs: &[Input], map: impl Fn(&Input) -> Result<String, ERR>) -> Result<Vec<String>, ERR> {
    inputs.into_iter().map(map).collect()
}

fn rayon(
    inputs: &[Input],
    map: impl Fn(&Input) -> Result<String, ERR> + Sync + Send,
) -> Result<Vec<String>, ERR> {
    use rayon::iter::*;
    inputs.into_par_iter().map(map).collect()
}

fn orx(
    inputs: &[Input],
    map: impl Fn(&Input) -> Result<String, ERR> + Sync + Clone,
) -> Result<Vec<String>, ERR> {
    use orx_parallel::*;
    inputs
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .map(map)
        .collect_result()
}

fn run(c: &mut Criterion) {
    let treatments = [N_EARLY, N_MIDDLE, N_LATE, N_NEVER];

    let mut group = c.benchmark_group("collect_result_arbitrary");

    for n_when in &treatments {
        let input = inputs(N, *n_when);
        let expected = seq(&input, map_input_to_result);

        let n_when = match *n_when {
            N_EARLY => "error-early",
            N_MIDDLE => "error-in-the-middle",
            N_LATE => "error-late",
            N_NEVER => "error-never",
            _ => panic!("unhandled n-when"),
        };

        // group.bench_with_input(BenchmarkId::new("seq", n_when), n_when, |b, _| {
        //     assert_eq!(&expected, &seq(&input, map_input_to_result));
        //     b.iter(|| seq(black_box(&input), map_input_to_result))
        // });

        group.bench_with_input(BenchmarkId::new("rayon", n_when), n_when, |b, _| {
            assert_eq!(&expected, &rayon(&input, map_input_to_result));
            b.iter(|| rayon(black_box(&input), map_input_to_result))
        });

        group.bench_with_input(BenchmarkId::new("orx", n_when), n_when, |b, _| {
            // assert_eq!(&expected, &orx(&input, map_input_to_result));
            b.iter(|| orx(black_box(&input), map_input_to_result))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
