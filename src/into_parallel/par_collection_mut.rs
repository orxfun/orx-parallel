use crate::infallible::{ParIter, xap_variants::Id};
use crate::into_parallel::par_collection::ParCollection;
use crate::runner::default_runner;
use orx_concurrent_iter::ConcurrentCollectionMut;

/// A collection from which a mutable parallel iterator can be created repeatedly
/// using `par_mut()` method.
///
/// Sequential counterpart: `iter_mut()`.
pub trait ParCollectionMut: ConcurrentCollectionMut + ParCollection {
    /// Returns a parallel iterator over mutable references to collection items.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut values = vec![1, 2, 3, 4];
    ///
    /// ParCollectionMut::par_mut(&mut values).for_each(|x| *x *= 2);
    /// assert_eq!(values, vec![2, 4, 6, 8]);
    ///
    /// // alternatively
    /// values.par_mut().for_each(|x| *x *= 2);
    /// assert_eq!(values, vec![4, 8, 12, 16]);
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

impl<X> ParCollectionMut for X where X: ConcurrentCollectionMut + ParCollection {}
