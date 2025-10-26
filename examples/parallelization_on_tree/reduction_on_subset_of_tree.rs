use crate::run_utils::timed;
use orx_parallel::*;

type Node = crate::node::Node<String>;

pub fn run(root: &Node) {
    println!("\n\n\n\n");
    println!(
        r#"# REDUCTION ON SUBSET OF THE TREE
        
In the previous examples we used "queue.extend" method to dynamically add children
to the queue.

However, this method requires the children to implement 'ExactSizeIterator'.
When we don't have pre-allocated children, or when we apply a filter on these children,
we cannot always satisfy this requirement.

In these cases,

* we can use `queue.push(child)` to add children one-by-one; or
* we can collect children into a vec and then use `queue.extend(children_vec)` to add them
  together.

`queue.push` approach has the following pros and cons:
* (+) makes new children available as soon as available.
* (+) does not require allocation.
* (-) might have greater parallelization overhead.

`queue.extend` approach has the following pros and cons:
* (+) will have the minimum parallelization overhead.
* (-) requires allocation for processing each node.

These are a couple of recommendations, we can use `push` and `extend` methods in a different
way to optimize our use case.
    "#
    );

    println!("\n\n\n\n# REDUCTION ON SUBSET OF THE TREE");
    let log = |sum: u64| println!("  sum = {sum}");

    timed("sequential", || sequential(root), log);

    timed("push_orx_rec", || push_orx_rec(root), log);
    timed(
        "collect_extend_orx_rec",
        || collect_extend_orx_rec(root),
        log,
    );

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

/// # orx-parallel: parallel recursive iterator with unknown length
///
/// Since we do not know how many children to add ahead-of-time, we
/// don't have an ExactSizeIterator. Therefore, instead of `queue.extend`,
/// we use `queue.push` to add new children.
///
/// * (+) makes new children available as soon as available.
/// * (+) does not require allocation.
/// * (-) might have greater parallelization overhead.
fn push_orx_rec(root: &Node) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node, queue: &Queue<&'b Node>) {
        for child in node.children.iter().filter(filter) {
            queue.push(child);
        }
    }

    [root].into_par_rec(extend).map(compute).sum()
}

/// # orx-parallel: parallel recursive iterator with unknown length
///
/// Alternatively, we can collect children in a vector and then call
/// `queue.extend` to add the new children.
///
/// * (+) will have the minimum parallelization overhead.
/// * (-) requires allocation for processing each node.
fn collect_extend_orx_rec(root: &Node) -> u64 {
    fn extend<'a, 'b>(node: &'a &'b Node, queue: &Queue<&'b Node>) {
        let children: Vec<_> = node.children.iter().filter(filter).collect();
        queue.extend(children);
    }

    [root].into_par_rec(extend).map(compute).sum()
}
