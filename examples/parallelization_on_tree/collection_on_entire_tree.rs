use crate::run_utils::timed;
use orx_parallel::*;

type Node = crate::node::Node<String>;

pub fn run(root: &Node) {
    println!("\n\n\n\n");
    println!(
        r#"# COLLECTION ON ENTIRE TREE
        
This example is almost the same as the "reduction" example.

The only difference is that instead of computing the some of mapped values,
we collect all mapped values in a vector.

This demonstrates the fact that a "parallel recursive iterator" is nothing but
a "parallel iterator" with access to all `ParIter` methods.

In order to change the computation from reduction to collection,
all we need to do is to change

[root].into_par_rec(extend).map(compute).sum()

into

[root].into_par_rec(extend).map(compute).collect()
    "#
    );

    let log = |vec: Vec<u64>| println!("  collection-len = {:?}", vec.len());

    timed("sequential", || sequential(root), log);
    timed("orx_rec", || orx_rec(root), log);
    timed("orx_rec_linearized", || orx_rec_linearized(root), log);
    timed("orx_rec_exact", || orx_rec_exact(root), log);

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

/// # orx-parallel: parallel recursive iterator with unknown length
///
/// Here we parallelize by providing the `extend` function.
///
/// Although we don't use it here, please consider `chunk_size`
/// optimization depending on the data whenever necessary. This might
/// be more important in non-linear data structures compared to linear
/// due to the dynamic nature of iteration.
fn orx_rec(root: &Node) -> Vec<u64> {
    fn extend<'a>(node: &&'a Node, queue: &Queue<&'a Node>) {
        queue.extend(&node.children);
    }

    [root].into_par_rec(extend).map(compute).collect()
}

/// # orx-parallel: parallel recursive iterator with unknown length
///
/// Here we parallelize by providing the `extend` function.
///
/// However, rather than parallel processing over a dynamic recursive
/// input, the iterator first flattens the tasks with the `linearize`
/// call and then operates on it as if it is over a linear data structure.
fn orx_rec_linearized(root: &Node) -> Vec<u64> {
    fn extend<'a>(node: &&'a Node, queue: &Queue<&'a Node>) {
        queue.extend(&node.children);
    }

    [root]
        .into_par_rec(extend)
        .linearize()
        .map(compute)
        .collect()
}

/// # orx-parallel: parallel recursive iterator with exact length
///
/// Here we parallelize by providing the `extend` function.
/// Further, we precompute the total number of children and provide it while creating
/// the parallel iterator. This is helpful to optimize parallel execution whenever
/// it is available and cheap to compute.
///
/// Good thing, we can also count the number of nodes in parallel.
///
/// On the other hand, it is good not to keep the chunk size too large in a recursive
/// iterator, as we limit it to 32 in the following.
fn orx_rec_exact(root: &Node) -> Vec<u64> {
    fn extend<'a>(node: &&'a Node, queue: &Queue<&'a Node>) {
        queue.extend(&node.children);
    }

    let num_nodes = [root].into_par_rec(extend).count();

    [root]
        .into_par_rec_exact(extend, num_nodes)
        .chunk_size(32)
        .map(compute)
        .collect()
}
