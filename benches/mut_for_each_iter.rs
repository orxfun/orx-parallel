use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Data {
    name: String,
    number: usize,
}

fn to_output(idx: usize) -> Data {
    let name = idx.to_string();
    let number = idx;
    Data { name, number }
}

fn inputs(len: usize) -> HashMap<usize, Data> {
    (0..len).map(|i| (i, to_output(i))).collect()
}

fn fibonacci(n: usize) -> usize {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn filter(data: &&mut Data) -> bool {
    !data.name.starts_with('3')
}

fn update(data: &mut Data) {
    for _ in 0..10 {
        let increment = fibonacci(data.number % 100) % 10;
        data.number += increment
    }
}

fn seq<'a>(inputs: impl Iterator<Item = &'a mut Data>) {
    inputs.filter(filter).for_each(update);
}

fn rayon<'a>(inputs: impl Iterator<Item = &'a mut Data> + Send) {
    use rayon::iter::{ParallelBridge, ParallelIterator};
    inputs.par_bridge().filter(filter).for_each(update);
}

fn orx<'a>(inputs: impl Iterator<Item = &'a mut Data>) {
    use orx_parallel::*;
    inputs.iter_into_par().filter(filter).for_each(update);
}

fn run(c: &mut Criterion) {
    let treatments = [65_536 * 2];

    let mut group = c.benchmark_group("mut_for_each_iter");

    for n in &treatments {
        let input = inputs(*n);
        let mut expected = input.clone();
        seq(expected.values_mut());
        let expected = expected;

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            let mut input = input.clone();
            seq(input.values_mut());
            assert_eq!(expected, input);
            b.iter(|| seq(input.values_mut()))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            let mut input = input.clone();
            rayon(input.values_mut());
            assert_eq!(expected, input);
            b.iter(|| rayon(input.values_mut()))
        });

        group.bench_with_input(BenchmarkId::new("orx", n), n, |b, _| {
            let mut input = input.clone();
            orx(input.values_mut());
            assert_eq!(expected, input);
            b.iter(|| orx(input.values_mut()))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
