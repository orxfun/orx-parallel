use crate::infallible::{ParIter, xap_variants::Id};
use crate::into_parallel::par_collection::ParCol;
use crate::runner::default_runner;
use orx_concurrent_iter::ConcurrentCollectionMut;

/// Adds `.par_mut()` to concurrent mutable collections.
///
/// Sequential counterpart: mutable iteration methods such as `iter_mut()`.
pub trait ParColMut: ConcurrentCollectionMut + ParCol {
    /// Returns a parallel iterator over mutable references to collection items.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut values = vec![1, 2, 3, 4];
    /// ParColMut::par_mut(&mut values).for_each(|x| *x *= 2);
    ///
    /// assert_eq!(values, vec![2, 4, 6, 8]);
    /// ```
    fn par_mut(&mut self) -> ParIter<Self::IterMut<'_>, Id<&mut Self::Item>> {
        ParIter::new(
            self.con_iter_mut(),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<X> ParColMut for X where X: ConcurrentCollectionMut + ParCol {}
