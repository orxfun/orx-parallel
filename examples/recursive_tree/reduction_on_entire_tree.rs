use crate::run_utils::timed;
use orx_parallel::*;
use std::sync::atomic::{AtomicU64, Ordering};

type Node = crate::node::Node<String>;

pub fn run(root: &Node) {
    println!("\n\n\n\n");
    println!(
        r#"# REDUCTION ON ENTIRE TREE

// orx_rec: recursive
[root].into_par_recursive(|node| &node.children).map(compute).sum()

// orx_rec_linearized: recursive to linearize, then regular parallel iter
let linearized: Vec<_> = [root].into_par_recursive(|node| &node.children).collect();
linearized.into_par().map(compute).sum()
    "#
    );

    let log = |sum: u64| println!("  sum = {sum}");

    timed("sequential", || sequential(root), log);
    timed("rayon", || rayon(root), log);
    timed("orx_rec", || orx_rec(root), log);
    timed("orx_rec_linearized", || orx_rec_linearized(root), log);

    println!();
}

/// Just a demo computation we perform for each node.
fn compute(node: &Node) -> u64 {
    crate::run_utils::compute(node.data.parse::<u64>().unwrap())
}

/// # sequential
///
/// This is a recursive sequential implementation to compute and reduce values of
/// all nodes descending from the root.
fn sequential(root: &Node) -> u64 {
    fn seq_compute_node(node: &Node) -> u64 {
        let node_value = compute(node);
        let child_values = node.children.iter().map(seq_compute_node);
        node_value + child_values.sum::<u64>()
    }

    seq_compute_node(root)
}

/// # rayon: defining the computation with rayon's scoped threads.
pub fn rayon(root: &Node) -> u64 {
    fn process_node<'scope>(sum: &'scope AtomicU64, node: &'scope Node, s: &rayon::Scope<'scope>) {
        for child in &node.children {
            s.spawn(move |s| {
                process_node(sum, child, s);
            });
        }
        let node_value = compute(node);
        sum.fetch_add(node_value, Ordering::Relaxed);
    }

    let sum = AtomicU64::new(0);
    rayon::in_place_scope(|s| {
        process_node(&sum, root, s);
    });
    sum.into_inner()
}

/// # orx-parallel: parallel recursive iterator
fn orx_rec(root: &Node) -> u64 {
    [root]
        .into_par_recursive(|node| &node.children)
        .map(compute)
        .sum()
}

/// # orx-parallel: parallel recursive iterator with linearization
///
/// Alternatively, we can collect children in a vector and then perform
/// parallel computation on linearized inputs.
fn orx_rec_linearized(root: &Node) -> u64 {
    let linearized: Vec<_> = [root].into_par_recursive(|node| &node.children).collect();
    linearized.into_par().map(compute).sum()
}
