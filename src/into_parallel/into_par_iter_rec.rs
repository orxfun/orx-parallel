use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;

/// Converts dynamically expanding recursive structures into an infallible parallel iterator.
///
/// Unlike flat sources (such as slices or ranges), recursive workloads discover new items
/// while they are being processed. This trait is useful when each item can produce additional
/// items, such as traversing a tree from its root(s).
pub trait IntoParIterRecursive
where
    Self: IntoIterator,
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
    /// let mut visited: Vec<_> = [0usize]
    ///     .into_par_recursive(|node| children[*node].iter().copied())
    ///     .map(|x| 2 * x + 1)
    ///     .collect();
    ///
    /// // Traversal order may vary in parallel, so compare as a set via sorting.
    /// visited.sort();
    /// assert_eq!(visited, vec![1, 3, 5, 7, 9, 11]);
    /// ```
    fn into_par_recursive<I, F>(
        self,
        extend: F,
    ) -> ParIter<ConcurrentRecursiveIter<I, F>, Id<Self::Item>>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync;
}

impl<X> IntoParIterRecursive for X
where
    X: IntoIterator,
    X::Item: Send,
{
    fn into_par_recursive<I, F>(
        self,
        extend: F,
    ) -> ParIter<ConcurrentRecursiveIter<I, F>, Id<Self::Item>>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync,
    {
        let iter = ConcurrentRecursiveIter::new(self, extend, None, None);
        ParIter::new(iter, Id::new(), default_runner(), Default::default())
    }
}
