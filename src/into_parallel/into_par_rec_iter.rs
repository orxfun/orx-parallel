use crate::infallible::ParRecIter;
use crate::infallible::xap_variants::Id;
use crate::runner::default_runner;

/// Creates an infallible parallel iterator for dynamically expanding recursive workloads.
///
/// Unlike flat sources such as slices or ranges, recursive workloads discover new items while
/// existing items are being processed. `initial_elements` provides the starting frontier, and
/// `extend` is called for each visited item to produce its children or follow-up work.
///
/// Despite parallel execution, recursive traversal can be deterministic. With
/// [`IterationOrder::Ordered`] (the default), order-sensitive operations use breadth-first order,
/// level by level and left-to-right following input and child generation order.
///
/// # Example
///
/// ```
/// use orx_parallel::*;
///
/// // A small rooted tree represented as adjacency lists.
/// // Node 0 is the root.
/// let children: Vec<Vec<usize>> = vec![
///     vec![1, 2], // children of 0
///     vec![3, 4], // children of 1
///     vec![5],    // children of 2
///     vec![],
///     vec![],
///     vec![],
/// ];
///
/// let visited: Vec<_> = par_recursive([0usize], |node| children[*node].iter().copied())
///     .map(|x| 2 * x + 1)
///     .collect();
///
/// // Ordered traversal is deterministic and breadth-first by default.
/// assert_eq!(visited, vec![1, 3, 5, 7, 9, 11]);
/// ```
///
/// [`IterationOrder::Ordered`]: crate::IterationOrder::Ordered
pub fn par_recursive<I, C, F>(initial_elements: I, extend: F) -> ParRecIter<I, Id<I::Item>, C, F>
where
    I: IntoIterator,
    C: IntoIterator<Item = I::Item>,
    F: Fn(&I::Item) -> C + Send + Sync,
{
    ParRecIter::new(
        initial_elements,
        Id::new(),
        default_runner(),
        Default::default(),
        extend,
    )
}
