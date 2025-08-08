use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::IntoParallelIterator;

const SEED: u64 = 5426;
const FIB_UPPER_BOUND: u32 = 29;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Out1 {
    name: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Out2 {
    name: String,
    number: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Out3 {
    out2: Out2,
    fib: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Out4 {
    a: String,
    b: char,
    fib: u32,
}

fn map1(idx: &usize) -> Out1 {
    let idx = *idx;
    let prefix = match idx % 7 {
        0 => "zero-",
        3 => "three-",
        _ => "sth-",
    };
    let fib = fibonacci(&(idx as u32 % FIB_UPPER_BOUND));
    let name = format!("{}-fib-{}", prefix, fib);
    Out1 { name }
}

fn filter1(output: &Out1) -> bool {
    let last_char = output.name.chars().last().unwrap();
    let last_digit: u32 = last_char.to_string().parse().unwrap();
    last_digit < 4
}

fn map2(input: Out1) -> Out2 {
    let number = (FIB_UPPER_BOUND + input.name.len() as u32).saturating_sub(10);
    let number = fibonacci(&(number & FIB_UPPER_BOUND));
    Out2 {
        name: number.to_string(),
        number,
    }
}

fn filter2(output: &Out2) -> bool {
    output.number % 2 == 0 || output.name.contains('0')
}

fn map3(input: Out2) -> Out3 {
    let fib = fibonacci(&input.number);
    Out3 { out2: input, fib }
}

fn map4(input: Out3) -> Out4 {
    let a = format!("{}!", input.out2.name);
    let b = input.out2.name.chars().next().unwrap();
    let fib = fibonacci(&((input.out2.number * 7) % FIB_UPPER_BOUND));
    Out4 { a, b, fib }
}

fn filter4(output: &Out4) -> bool {
    output.a.len() == 5 || output.b == 'x' || output.fib % 2 == 1
}

fn reduce(x: Out4, y: Out4) -> Out4 {
    let a = match x.fib % 2 == 0 {
        true => x.a,
        false => y.a,
    };
    let b = match y.fib % 2 == 0 {
        true => x.b,
        false => y.b,
    };
    let fib = x.fib + y.fib;
    Out4 { a, b, fib }
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

fn inputs(len: usize) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| rng.random_range(0..FIB_UPPER_BOUND) as usize)
        .collect()
}

fn seq(inputs: &[usize]) -> Option<Out4> {
    inputs
        .iter()
        .map(map1)
        .filter(filter1)
        .map(map2)
        .filter(filter2)
        .map(map3)
        .map(map4)
        .filter(filter4)
        .reduce(reduce)
}

fn rayon(inputs: &[usize]) -> Option<Out4> {
    use rayon::iter::ParallelIterator;
    inputs
        .into_par_iter()
        .map(map1)
        .filter(filter1)
        .map(map2)
        .filter(filter2)
        .map(map3)
        .map(map4)
        .filter(filter4)
        .reduce_with(reduce)
}

fn orx(inputs: &[usize]) -> Option<Out4> {
    inputs
        .into_par()
        .map(map1)
        .filter(filter1)
        .map(map2)
        .filter(filter2)
        .map(map3)
        .map(map4)
        .filter(filter4)
        .reduce(reduce)
}

fn run(c: &mut Criterion) {
    let treatments = [65_536 * 2];

    let mut group = c.benchmark_group("collect_long_chain");

    for n in &treatments {
        let input = inputs(*n);
        let expected = seq(&input);

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            assert_eq!(&expected, &seq(&input));
            b.iter(|| seq(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            assert_eq!(&expected, &rayon(&input));
            b.iter(|| rayon(black_box(&input)))
        });

        group.bench_with_input(BenchmarkId::new("orx", n), n, |b, _| {
            assert_eq!(&expected, &orx(&input));
            b.iter(|| orx(black_box(&input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
