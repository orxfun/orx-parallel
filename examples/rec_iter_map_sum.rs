use orx_concurrent_recursive_iter::Queue;
use orx_parallel::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

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

    fn seq_sum_fib(&self) -> u64 {
        self.value.iter().map(|x| fibonacci(*x)).sum::<u64>()
            + self.children.iter().map(|x| x.seq_sum_fib()).sum::<u64>()
    }
}

fn par_rec(roots: &[Node]) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node, queue: &Queue<&'b Node>) {
        queue.extend(&node.children);
    }

    let count: usize = roots.iter().map(|x| x.seq_num_nodes()).sum();

    let runner = DefaultRunner::default().with_diagnostics();

    roots
        .into_par_rec_exact(extend, count)
        .with_runner(runner)
        .map(|x| x.value.iter().map(|x| fibonacci(*x)).sum::<u64>())
        .sum()
}

fn par_rec_eager(roots: &[Node]) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node, queue: &Queue<&'b Node>) {
        queue.extend(&node.children);
    }

    let runner = DefaultRunner::default().with_diagnostics();

    roots
        .into_par_rec(extend)
        .into_eager()
        .with_runner(runner)
        .map(|x| x.value.iter().map(|x| fibonacci(*x)).sum::<u64>())
        .sum()
}

fn main() {
    println!("\n\n");
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let roots = vec![
        Node::new(500, &mut rng),
        Node::new(200, &mut rng),
        Node::new(400, &mut rng),
    ];

    let count: usize = roots.iter().map(|x| x.seq_num_nodes()).sum();
    println!("Tree contains {count} nodes");

    let expected: u64 = roots.iter().map(|x| x.seq_sum_fib()).sum();

    let sum_fib = par_rec_eager(&roots);
    assert_eq!(sum_fib, expected);
    println!("Sum of Fibonacci of node values is {sum_fib}");

    let sum_fib = par_rec(&roots);
    assert_eq!(sum_fib, expected);
    println!("Sum of Fibonacci of node values is {sum_fib}");
}
