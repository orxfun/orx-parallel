use crate::tree::Tree;
use clap::Parser;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::sync::OnceLock;

mod collection_on_entire_tree;
mod node;
mod reduction_on_entire_tree;
mod reduction_on_subset_of_tree;
mod run_utils;
mod tree;

#[derive(Parser, Debug)]
struct Args {
    /// Amount of work (num times Fibonacci will be repeated).
    #[arg(long, default_value_t = 10)]
    amount_of_work: usize,
}

pub fn amount_of_work() -> &'static usize {
    static WORK: OnceLock<usize> = OnceLock::new();
    WORK.get_or_init(|| Args::parse().amount_of_work)
}

fn main() {
    let num_nodes = 100_000;
    let out_degree = 0..100;

    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let data = |idx: usize| idx.to_string();
    let root = Tree::new(num_nodes, out_degree, data, &mut rng);

    println!("\nOne path from root to a leaf as an example");
    let mut next_node = Some(&root);
    let mut i = 0;
    while let Some(node) = next_node {
        let indent: String = (0..i).map(|_| '*').collect();
        println!("{indent}{node:?}");
        i += 1;
        next_node = node.children.iter().max_by_key(|x| x.children.len());
    }

    println!("\nTotal number of nodes = {}", root.num_nodes());

    reduction_on_entire_tree::run(&root);
    collection_on_entire_tree::run(&root);
    reduction_on_subset_of_tree::run(&root);
}
