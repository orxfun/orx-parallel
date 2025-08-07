use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

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

fn inputs(len: usize) -> Vec<Data> {
    (0..len).map(to_output).collect()
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

fn seq(inputs: &mut Vec<Data>) {
    inputs.iter_mut().filter(filter).for_each(update);
}

fn rayon(inputs: &mut Vec<Data>) {
    use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
    inputs.par_iter_mut().filter(filter).for_each(update);
}

fn orx(inputs: &mut Vec<Data>) {
    use orx_parallel::*;
    inputs.into_par().filter(filter).for_each(update);
}

fn run(c: &mut Criterion) {
    let treatments = [65_536 * 2];

    let mut group = c.benchmark_group("mut_for_each_slice");

    for n in &treatments {
        let input = inputs(*n);
        let mut expected = input.clone();
        seq(&mut expected);

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            let mut input = input.clone();
            seq(&mut input);
            assert_eq!(expected, input);
            b.iter(|| seq(black_box(&mut input)))
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            let mut input = input.clone();
            seq(&mut input);
            assert_eq!(expected, input);
            b.iter(|| rayon(black_box(&mut input)))
        });

        group.bench_with_input(BenchmarkId::new("orx", n), n, |b, _| {
            let mut input = input.clone();
            seq(&mut input);
            assert_eq!(expected, input);
            b.iter(|| orx(black_box(&mut input)))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
