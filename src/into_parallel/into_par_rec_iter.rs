use crate::infallible::ParRecIter;
use crate::infallible::xap_variants::Id;
use crate::runner::default_runner;

/// Converts dynamically expanding recursive structures into an infallible parallel iterator.
///
/// Unlike flat sources (such as slices or ranges), recursive workloads discover new items
/// while they are being processed. This trait is useful when each item can produce additional
/// items, such as traversing a tree from its root(s).
///
/// Despite parallel execution, recursive traversal can be deterministic. With
/// [`IterationOrder::Ordered`] (the default), order-sensitive operations use breadth-first order,
/// level by level and left-to-right following input and child generation order.
///
/// [`IterationOrder::Ordered`]: crate::IterationOrder::Ordered
pub trait IntoParRec
where
    Self: IntoIterator + Sized,
    Self::Item: Send,
{
    /// Creates a parallel recursive iterator using `extend` to discover children.
    ///
    /// This differs from regular `into_par()` in that the final work set is not known upfront.
    /// Instead, each visited item can contribute more items dynamically through `extend`.
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
    /// // Start from the root and expand recursively.
    /// let visited: Vec<_> = [0usize]
    ///     .into_par_rec(|node| children[*node].iter().copied())
    ///     .map(|x| 2 * x + 1)
    ///     .collect();
    ///
    /// // Ordered traversal is deterministic and breadth-first by default.
    /// assert_eq!(visited, vec![1, 3, 5, 7, 9, 11]);
    /// ```
    fn into_par_rec<I, F>(self, extend: F) -> ParRecIter<Self, Id<Self::Item>, I, F>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync;
}

impl<S> IntoParRec for S
where
    S: IntoIterator + Sized,
    S::Item: Send,
{
    fn into_par_rec<I, F>(self, extend: F) -> ParRecIter<Self, Id<Self::Item>, I, F>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync,
    {
        ParRecIter::new(
            self,
            Id::new(),
            default_runner(),
            Default::default(),
            extend,
        )
    }
}
