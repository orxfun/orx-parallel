use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{
    hint::black_box,
    sync::atomic::{AtomicU64, Ordering},
};

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

    fn seq_sum_fib(&self, work: usize) -> u64 {
        self.value.iter().map(|x| fibonacci(*x, work)).sum::<u64>()
            + self
                .children
                .iter()
                .map(|x| x.seq_sum_fib(work))
                .sum::<u64>()
    }
}

// alternatives

fn seq(roots: &[Node], work: usize) -> u64 {
    roots.iter().map(|x| x.seq_sum_fib(work)).sum()
}

fn rayon(roots: &[Node], work: usize) -> u64 {
    use rayon::iter::*;
    fn process_node<'scope>(
        work: usize,
        sum: &'scope AtomicU64,
        node: &'scope Node,
        s: &rayon::Scope<'scope>,
    ) {
        for child in &node.children {
            s.spawn(move |s| {
                process_node(work, sum, child, s);
            });
        }
        let val = node.value.par_iter().map(|x| fibonacci(*x, work)).sum();
        sum.fetch_add(val, Ordering::Relaxed);
    }

    let sum = AtomicU64::new(0);
    rayon::in_place_scope(|s| {
        for root in roots {
            process_node(work, &sum, root, s);
        }
    });
    sum.into_inner()
}

fn orx_lazy_unknown_chunk1024(roots: &[Node], work: usize) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    roots
        .into_par_rec(extend)
        .chunk_size(1024)
        .map(|x| x.value.iter().map(|x| fibonacci(*x, work)).sum::<u64>())
        .sum()
}

fn orx_lazy_exact(roots: &[Node], work: usize, num_nodes: usize) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    roots
        .into_par_rec_exact(extend, num_nodes)
        .map(|x| x.value.iter().map(|x| fibonacci(*x, work)).sum::<u64>())
        .sum()
}

fn orx_eager(roots: &[Node], work: usize) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    roots
        .into_par_rec(extend)
        .into_eager()
        .map(|x| x.value.iter().map(|x| fibonacci(*x, work)).sum::<u64>())
        .sum()
}

fn run(c: &mut Criterion) {
    let treatments = [1, 10, 25];
    let mut group = c.benchmark_group("rec_iter_map_sum");
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let roots = vec![
        Node::new(5000, &mut rng),
        Node::new(2000, &mut rng),
        Node::new(4000, &mut rng),
    ];

    let num_nodes: usize = roots.iter().map(|x| x.seq_num_nodes()).sum();

    for work in &treatments {
        let expected = seq(&roots, *work);

        group.bench_with_input(BenchmarkId::new("seq", work), work, |b, _| {
            assert_eq!(&expected, &seq(&roots, *work));
            b.iter(|| seq(&roots, *work))
        });

        group.bench_with_input(BenchmarkId::new("rayon", work), work, |b, _| {
            assert_eq!(&expected, &rayon(&roots, *work));
            b.iter(|| rayon(&roots, *work))
        });

        group.bench_with_input(
            BenchmarkId::new("orx_lazy_unknown_chunk1024", work),
            work,
            |b, _| {
                assert_eq!(&expected, &orx_lazy_unknown_chunk1024(&roots, *work));
                b.iter(|| orx_lazy_unknown_chunk1024(&roots, *work))
            },
        );

        group.bench_with_input(BenchmarkId::new("orx_lazy_exact", work), work, |b, _| {
            assert_eq!(&expected, &orx_lazy_exact(&roots, *work, num_nodes));
            b.iter(|| orx_lazy_exact(&roots, *work, num_nodes))
        });

        group.bench_with_input(BenchmarkId::new("orx_eager", work), work, |b, _| {
            assert_eq!(&expected, &orx_eager(&roots, *work));
            b.iter(|| orx_eager(&roots, *work))
        });
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
