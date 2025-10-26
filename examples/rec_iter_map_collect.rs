use orx_concurrent_recursive_iter::Queue;
use orx_parallel::*;
use orx_split_vec::SplitVec;
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

    fn seq(&self, numbers: &mut Vec<u64>) {
        numbers.extend(self.value.iter().map(|x| fibonacci(*x)));
        for c in &self.children {
            c.seq(numbers);
        }
    }
}

fn par_rec(roots: &[Node]) -> SplitVec<u64> {
    fn extend<'a, 'b>(node: &'a &'b Node, queue: &Queue<&'b Node>) {
        queue.extend(&node.children);
    }
    let count: usize = roots.iter().map(|x| x.seq_num_nodes()).sum();

    let runner = DefaultRunner::default().with_diagnostics();

    roots
        .into_par_rec_exact(extend, count)
        .with_runner(runner)
        .flat_map(|x| x.value.iter().map(|x| fibonacci(*x)))
        .collect()
}

fn par_rec_eager(roots: &[Node]) -> SplitVec<u64> {
    fn extend<'a, 'b>(node: &'a &'b Node, queue: &Queue<&'b Node>) {
        queue.extend(&node.children);
    }

    let runner = DefaultRunner::default().with_diagnostics();

    roots
        .into_par_rec(extend)
        .into_eager()
        .with_runner(runner)
        .flat_map(|x| x.value.iter().map(|x| fibonacci(*x)))
        .collect()
}

fn main() {
    println!("\n\n");
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let roots = vec![
        Node::new(5000, &mut rng),
        Node::new(2000, &mut rng),
        Node::new(4000, &mut rng),
    ];

    let count: usize = roots.iter().map(|x| x.seq_num_nodes()).sum();
    println!("Tree contains {count} nodes");

    let mut expected = vec![];
    for root in &roots {
        root.seq(&mut expected);
    }
    expected.sort();

    println!("\n\n\n\n# par_rec");
    let mut result = par_rec(&roots).to_vec();
    result.sort();
    assert_eq!(result, expected);

    println!("\n\n\n\n# par_rec_eager");
    let mut result = par_rec_eager(&roots).to_vec();
    result.sort();
    assert_eq!(result, expected);
}
