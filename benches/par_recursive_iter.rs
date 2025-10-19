use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{
    hint::black_box,
    sync::atomic::{AtomicU64, Ordering},
};

struct Node {
    value: u64,
    children: Vec<Node>,
}

fn fibonacci(n: u64) -> u64 {
    // let n = n % 42; // let's not overflow
    (0..100)
        .map(|i| {
            let n = i + n;
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

impl Node {
    fn new(rng: &mut impl Rng, value: u64) -> Self {
        let num_children = match value {
            0 => 0,
            n => rng.random_range(0..(n as usize)),
        };
        let children = (0..num_children)
            .map(|i| Self::new(rng, i as u64))
            .collect();
        Self { value, children }
    }

    fn seq_num_nodes(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|node| node.seq_num_nodes())
            .sum::<usize>()
    }

    fn seq_sum_fib(&self) -> u64 {
        fibonacci(self.value) + self.children.iter().map(|x| x.seq_sum_fib()).sum::<u64>()
    }
}

fn seq(root: &Node) -> u64 {
    root.seq_sum_fib()
}

fn rayon(root: &Node) -> u64 {
    fn process_node<'scope>(sum: &'scope AtomicU64, node: &'scope Node, s: &rayon::Scope<'scope>) {
        for child in &node.children {
            s.spawn(|s| {
                process_node(sum, child, s);
            });
        }
        let val = fibonacci(node.value);
        sum.fetch_add(val, Ordering::Relaxed);
    }

    let sum = AtomicU64::new(0);
    rayon::in_place_scope(|s| {
        process_node(&sum, root, s);
    });
    sum.into_inner()
}

fn orx_lazy_unknown(root: &Node) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    [root]
        .into_par_rec(extend)
        // .chunk_size(1024 * 64)
        .map(|x| fibonacci(x.value))
        .sum()
}

fn orx_lazy_exact(root: &Node) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    let num_nodes = root.seq_num_nodes();

    [root]
        .into_par_rec_exact(extend, num_nodes)
        .map(|x| fibonacci(x.value))
        .sum()
}

fn orx_eager(root: &Node) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    [root]
        .into_par_rec(extend)
        .into_eager()
        .map(|x| fibonacci(x.value))
        .sum()
}

fn orx_static(root: &Node) -> u64 {
    fn add_tasks<'a>(tasks: &mut Vec<&'a Node>, node: &'a Node) {
        tasks.push(node);
        for child in &node.children {
            add_tasks(tasks, child);
        }
    }
    // let mut tasks = Vec::with_capacity(root.seq_num_nodes() + 1);
    let mut tasks = Vec::new();
    add_tasks(&mut tasks, root);
    tasks.par().map(|x| fibonacci(x.value)).sum()
}

fn iter(root: &Node) -> u64 {
    use orx_concurrent_iter::*;
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    let num_threads = 32;
    let iter = ConcurrentRecursiveIter::new(extend, [root]);
    let num_spawned = core::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(|| {
                // allow all threads to be spawned
                _ = num_spawned.fetch_add(1, Ordering::Relaxed);
                while num_spawned.load(Ordering::Relaxed) < num_threads {}

                // computation: parallel reduction
                let mut thread_sum = 0;
                let mut puller = iter.chunk_puller(1024);
                while let Some(chunk) = puller.pull() {
                    thread_sum += chunk.into_iter().map(|x| fibonacci(x.value)).sum::<u64>();
                }
                thread_sum
            }));
        }

        handles.into_iter().map(|x| x.join().unwrap()).sum()
    })
}

fn run(c: &mut Criterion) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let root = Node::new(&mut rng, 500);
    let n = &root.seq_num_nodes();
    let expected = root.seq_sum_fib();

    let mut group = c.benchmark_group("par_recursive_iter");

    group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
        assert_eq!(&expected, &seq(&root));
        b.iter(|| seq(black_box(&root)))
    });

    group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
        assert_eq!(&expected, &rayon(&root));
        b.iter(|| rayon(black_box(&root)))
    });

    // group.bench_with_input(BenchmarkId::new("orx_lazy_unknown", n), n, |b, _| {
    //     assert_eq!(&expected, &orx_lazy_unknown(&root));
    //     b.iter(|| orx_lazy_unknown(black_box(&root)))
    // });

    group.bench_with_input(BenchmarkId::new("orx_lazy_exact", n), n, |b, _| {
        assert_eq!(&expected, &orx_lazy_exact(&root));
        b.iter(|| orx_lazy_exact(black_box(&root)))
    });

    group.bench_with_input(BenchmarkId::new("orx_eager", n), n, |b, _| {
        assert_eq!(&expected, &orx_eager(&root));
        b.iter(|| orx_eager(black_box(&root)))
    });

    group.bench_with_input(BenchmarkId::new("orx_static", n), n, |b, _| {
        assert_eq!(&expected, &orx_static(&root));
        b.iter(|| orx_static(black_box(&root)))
    });

    group.bench_with_input(BenchmarkId::new("iter", n), n, |b, _| {
        assert_eq!(&expected, &iter(&root));
        b.iter(|| iter(black_box(&root)))
    });

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
