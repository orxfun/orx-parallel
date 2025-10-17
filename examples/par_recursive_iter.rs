use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;
use orx_parallel::{IntoParIterRec, ParIter};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::atomic::Ordering;

struct Node {
    value: u64,
    children: Vec<Node>,
}

fn fibonacci(n: u64) -> u64 {
    let n = n % 42; // let's not overflow
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
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

fn par_rec(root: &Node) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    [root]
        .into_par_rec(extend)
        .chunk_size(1024 * 1024)
        .num_threads(32)
        .map(|x| fibonacci(x.value))
        .sum()
}

fn iter(root: &Node) -> u64 {
    use orx_concurrent_iter::*;
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    let num_threads = 16;
    let chunk_size = 1024;
    let iter = ConcurrentRecursiveIter::new(extend, [root]);
    let num_spawned = core::sync::atomic::AtomicUsize::new(0);
    let num_handled = core::sync::atomic::AtomicUsize::new(0);

    std::thread::scope(|s| {
        let mut handles = vec![];
        for _ in 0..num_threads {
            handles.push(s.spawn(|| {
                // allow all threads to be spawned
                _ = num_spawned.fetch_add(1, Ordering::Relaxed);
                while num_spawned.load(Ordering::Relaxed) < num_threads {}

                // computation: parallel reduction
                let mut thread_sum = 0;
                let mut puller = iter.chunk_puller(chunk_size);
                while let Some(chunk) = puller.pull() {
                    thread_sum += chunk.into_iter().map(|x| fibonacci(x.value)).sum::<u64>();
                }

                thread_sum
            }));
        }

        handles.into_iter().map(|x| x.join().unwrap()).sum()
    })
}

fn main() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let root = Node::new(&mut rng, 550);

    // let par = [&root].into_par_rec(extend);
    // let count = par.count();
    // assert_eq!(count, root.seq_num_nodes());
    let count = root.seq_num_nodes();
    println!("Tree contains {count} nodes");

    let expected = root.seq_sum_fib();

    let sum_fib = par_rec(&root);
    assert_eq!(sum_fib, expected);
    println!("Sum of Fibonacci of node values is {sum_fib}");

    let sum_fib = iter(&root);
    assert_eq!(sum_fib, expected);
    println!("Sum of Fibonacci of node values is {sum_fib}");
}
