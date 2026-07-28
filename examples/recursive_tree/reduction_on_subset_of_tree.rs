use crate::run_utils::timed;
use orx_parallel::*;

type Node = crate::node::Node<String>;

pub fn run(root: &Node) {
    println!("\n\n\n\n");
    println!(
        r#"# REDUCTION ON SUBSET OF THE TREE
        
// orx_rec: recursive
[root].into_par_recursive(|node| node.children.iter().filter(filter)).map(compute).sum()

// orx_rec_linearized: recursive to linearize, then regular parallel iter
let linearized: Vec<_> = [root].into_par_recursive(|node| node.children.iter().filter(filter)).collect();
linearized.into_par().map(compute).sum()
    "#
    );

    println!("\n\n\n\n# REDUCTION ON SUBSET OF THE TREE");
    let log = |sum: u64| println!("  sum = {sum}");

    timed("sequential", || sequential(root), log);
    timed("orx_rec", || orx_rec(root), log);
    timed("orx_rec_linearized", || orx_rec_linearized(root), log);

    println!();
}

/// Just a demo computation we perform for each node.
fn compute(node: &Node) -> u64 {
    crate::run_utils::compute(node.data.parse::<u64>().unwrap())
}

fn filter(node: &&Node) -> bool {
    !node.data.parse::<u64>().unwrap().is_multiple_of(42)
}

/// # sequential
///
/// This is a recursive sequential implementation to compute and reduce values of
/// all nodes descending from the root.
fn sequential(root: &Node) -> u64 {
    fn seq_compute_node(node: &Node) -> u64 {
        let node_value = compute(node);
        let child_values = node.children.iter().filter(filter).map(seq_compute_node);
        node_value + child_values.sum::<u64>()
    }

    seq_compute_node(root)
}

/// # orx-parallel: parallel recursive iterator
fn orx_rec(root: &Node) -> u64 {
    [root]
        .into_par_recursive(|node| node.children.iter().filter(filter))
        .map(compute)
        .sum()
}

/// # orx-parallel: parallel recursive iterator with linearization
///
/// Alternatively, we can collect children in a vector and then perform
/// parallel computation on linearized inputs.
fn orx_rec_linearized(root: &Node) -> u64 {
    let linearized: Vec<_> = [root]
        .into_par_recursive(|node| node.children.iter().filter(filter))
        .collect();
    linearized.into_par().map(compute).sum()
}
