use orx_parallel::{IntoParIterRec, ParIter};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

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

    fn num_nodes(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|node| node.num_nodes())
            .sum::<usize>()
    }
}

fn main() {
    fn extend<'a, 'b>(node: &'a &'b Node) -> &'b [Node] {
        &node.children
    }

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let root = Node::new(&mut rng, 300);

    let par = [&root].into_par_rec(extend);
    let count = par.count();
    assert_eq!(count, root.num_nodes());
    println!("Tree contains {count} nodes");

    let par = [&root].into_par_rec(extend);
    let sum_fib = par.map(|x| fibonacci(x.value)).sum();
    assert_eq!(sum_fib, 4843403551);
    println!("Sum of Fibonacci of node values is {sum_fib}");
}
