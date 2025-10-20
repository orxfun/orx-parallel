use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use orx_concurrent_bag::ConcurrentBag;
use orx_parallel::*;
use orx_split_vec::SplitVec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{hint::black_box, sync::atomic::AtomicU64};

fn fibonacci(n: u64, work: usize) -> u64 {
    (7..(work + 7))
        .map(|j| {
            let n = black_box((n + j as u64) % 100);
            let mut a = 0;
            let mut b = 1;
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            a
        })
        .sum()
}

struct Node {
    value: Vec<u64>,
    children: Vec<Node>,
}

impl Node {
    fn new(mut n: u32, rng: &mut impl Rng) -> Self {
        let mut children = Vec::new();
        if n < 5 {
            for _ in 0..n {
                children.push(Node::new(0, rng));
            }
        } else {
            while n > 0 {
                let n2 = rng.random_range(0..=n);
                children.push(Node::new(n2, rng));
                n -= n2;
            }
        }
        Self {
            value: (0..rng.random_range(1..500))
                .map(|_| rng.random_range(0..40))
                .collect(),
            children,
        }
    }

    fn seq_num_nodes(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|node| node.seq_num_nodes())
            .sum::<usize>()
    }

    fn seq(&self, work: usize, numbers: &mut Vec<u64>) {
        numbers.extend(self.value.iter().map(|x| fibonacci(*x, work)));
        for c in &self.children {
            c.seq(work, numbers);
        }
    }
}

// alternatives

fn seq(roots: &[Node], work: usize) -> Vec<u64> {
    let mut result = vec![];
    for root in roots {
        root.seq(work, &mut result);
    }
    result
}

fn orx_lazy_unknown_chunk1024(roots: &[Node], work: usize) -> SplitVec<u64> {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    roots
        .into_par_rec(extend)
        .chunk_size(1024)
        .flat_map(|x| x.value.iter().map(|x| fibonacci(*x, work)))
        .collect()
}

fn orx_lazy_exact(roots: &[Node], work: usize, num_nodes: usize) -> SplitVec<u64> {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    roots
        .into_par_rec_exact(extend, num_nodes)
        .flat_map(|x| x.value.iter().map(|x| fibonacci(*x, work)))
        .collect()
}

fn orx_eager(roots: &[Node], work: usize) -> SplitVec<u64> {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    roots
        .into_par_rec(extend)
        .into_eager()
        .flat_map(|x| x.value.iter().map(|x| fibonacci(*x, work)))
        .collect()
}

fn run(c: &mut Criterion) {
    let treatments = [1, 10, 25];
    let mut group = c.benchmark_group("rec_iter_map_collect");
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let roots = vec![
        Node::new(5000, &mut rng),
        Node::new(2000, &mut rng),
        Node::new(4000, &mut rng),
    ];

    let num_nodes: usize = roots.iter().map(|x| x.seq_num_nodes()).sum();

    for work in &treatments {
        let mut expected = seq(&roots, *work);
        expected.sort();

        group.bench_with_input(BenchmarkId::new("seq", work), work, |b, _| {
            let mut result = seq(&roots, *work);
            result.sort();
            assert_eq!(&expected, &result);
            b.iter(|| seq(&roots, *work))
        });

        group.bench_with_input(BenchmarkId::new("orx_lazy_exact", work), work, |b, _| {
            let mut result = orx_lazy_exact(&roots, *work, num_nodes).to_vec();
            result.sort();
            assert_eq!(&expected, &result);
            b.iter(|| orx_lazy_exact(&roots, *work, num_nodes))
        });

        group.bench_with_input(
            BenchmarkId::new("orx_lazy_unknown_chunk1024", work),
            work,
            |b, _| {
                let mut result = orx_lazy_unknown_chunk1024(&roots, *work).to_vec();
                result.sort();
                assert_eq!(&expected, &result);
                b.iter(|| orx_lazy_unknown_chunk1024(&roots, *work))
            },
        );

        group.bench_with_input(BenchmarkId::new("orx_eager", work), work, |b, _| {
            let mut result = orx_eager(&roots, *work).to_vec();
            result.sort();
            assert_eq!(&expected, &result);
            b.iter(|| orx_eager(&roots, *work))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
