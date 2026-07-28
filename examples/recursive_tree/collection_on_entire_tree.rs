use crate::run_utils::timed;
use orx_parallel::*;

type Node = crate::node::Node<String>;

pub fn run(root: &Node) {
    println!("\n\n\n\n");
    println!(
        r#"# COLLECTION ON ENTIRE TREE
        
// orx_rec: recursive
[root].into_par_recursive(|node| &node.children).map(compute).collect()

// orx_rec_linearized: recursive to linearize, then regular parallel iter
let linearized: Vec<_> = [root].into_par_recursive(|node| &node.children).collect();
linearized.into_par().map(compute).collect()
    "#
    );

    let log = |vec: Vec<u64>| println!("  collection-len = {:?}", vec.len());

    timed("sequential", || sequential(root), log);
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
fn sequential(root: &Node) -> Vec<u64> {
    fn seq_compute_node(node: &Node, result: &mut Vec<u64>) {
        let node_value = compute(node);
        result.push(node_value);

        for child in &node.children {
            seq_compute_node(child, result);
        }
    }

    let mut result = vec![];
    seq_compute_node(root, &mut result);
    result
}

/// # orx-parallel: parallel recursive iterator
fn orx_rec(root: &Node) -> Vec<u64> {
    [root]
        .into_par_recursive(|node| &node.children)
        .map(compute)
        .collect()
}

/// # orx-parallel: parallel recursive iterator with linearization
///
/// Alternatively, we can collect children in a vector and then perform
/// parallel computation on linearized inputs.
fn orx_rec_linearized(root: &Node) -> Vec<u64> {
    let linearized: Vec<_> = [root].into_par_recursive(|node| &node.children).collect();
    linearized.into_par().map(compute).collect()
}
