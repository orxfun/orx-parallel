use crate::run_utils::timed;
use orx_parallel::*;
use std::sync::atomic::{AtomicU64, Ordering};

type Node = crate::node::Node<String>;

pub fn run(root: &Node) {
    println!("\n\n\n\n");
    println!(
        r#"# REDUCTION ON ENTIRE TREE
        
This example demonstrates parallel computation over a tree.
Unlike parallelization over linear data structures, we don't have access to all
input elements, or say tasks, ahead of time.

Instead, the new elements are added dynamically on the fly.
Therefore, these iterators are called "parallel recursive iterator"s.

In addition to an initial set of elements, a parallel recursive iterator is
created with an "extend" function which defines the recursive behavior.

In this example we create an iterator where elements are "&Node".
We define the "extend" function as follows:

fn extend<'a>(node: &&'a Node, queue: &Queue<&'a Node>) {{
    queue.extend(&node.children);
}}

While processing a particular "node", we add all its children to the "queue".
In this example, we use "queue.extend", later we will also use "queue.push".

This allows to express the parallel computation as simple as over a linear
data structure:

[root].into_par_rec(extend).map(compute).sum()
    "#
    );

    let log = |sum: u64| println!("  sum = {sum}");

    timed("sequential", || sequential(root), log);

    // rayon miri fails with:
    // Undefined Behavior: trying to retag from <84156795> for SharedReadWrite permission at alloc41643328[0x8],
    // but that tag does not exist in the borrow stack for this location
    #[cfg(not(miri))]
    timed("rayon", || rayon(root), log);

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
fn sequential(root: &Node) -> u64 {
    fn seq_compute_node(node: &Node) -> u64 {
        let node_value = compute(node);
        let child_values = node.children.iter().map(|x| seq_compute_node(x));
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

/// # orx-parallel: parallel recursive iterator with unknown length
///
/// Here we parallelize by providing the `extend` function.
///
/// Although we don't use it here, please consider `chunk_size`
/// optimization depending on the data whenever necessary. This might
/// be more important in non-linear data structures compared to linear
/// due to the dynamic nature of iteration.
fn orx_rec(root: &Node) -> u64 {
    fn extend<'a>(node: &&'a Node, queue: &Queue<&'a Node>) {
        queue.extend(&node.children);
    }

    [root].into_par_rec(extend).map(compute).sum()
}

/// # orx-parallel: parallel recursive iterator with unknown length
///
/// Here we parallelize by providing the `extend` function.
///
/// However, rather than parallel processing over a dynamic recursive
/// input, the iterator first flattens the tasks with the `linearize`
/// call and then operates on it as if it is over a linear data structure.
fn orx_rec_linearized(root: &Node) -> u64 {
    fn extend<'a>(node: &&'a Node, queue: &Queue<&'a Node>) {
        queue.extend(&node.children);
    }

    [root].into_par_rec(extend).linearize().map(compute).sum()
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
fn orx_rec_exact(root: &Node) -> u64 {
    fn extend<'a>(node: &&'a Node, queue: &Queue<&'a Node>) {
        queue.extend(&node.children);
    }

    let num_nodes = [root].into_par_rec(extend).count();

    [root]
        .into_par_rec_exact(extend, num_nodes)
        .chunk_size(32)
        .map(compute)
        .sum()
}
