use crate::{Params, computational_variants::Par, orch::DefaultOrchestrator};
use orx_concurrent_iter::ConcurrentDrainableOverSlice;
use std::ops::RangeBounds;

/// A type which can create a parallel draining iterator over any of its sub-slices.
///
/// * Created draining iterator takes out and returns all elements of the slice defined by the `range`.
/// * The slice defined by this `range` will be removed from the original collection.
///
/// If the iterator is dropped before being fully consumed, it drops the remaining removed elements.
///
/// If the complete range is provided (`..` or `0..self.len()`), self will remain empty.
///
/// # Examples
///
/// ```
/// use orx_parallel::*;
///
/// let mut v = vec![1, 2, 3];
/// let u: Vec<_> = v.par_drain(1..).num_threads(2).collect();
///
/// assert_eq!(v, &[1]);
/// assert_eq!(u, &[2, 3]);
/// ```
pub trait ParallelDrainableOverSlice: ConcurrentDrainableOverSlice {
    /// Creates a parallel draining iterator over the slice defined by the given `range`.
    ///
    /// * Created draining iterator takes out and returns all elements of the slice defined by the `range`.
    /// * The slice defined by this `range` will be removed from the original collection.
    ///
    /// If the iterator is dropped before being fully consumed, it drops the remaining removed elements.
    ///
    /// If the complete range is provided (`..` or `0..self.len()`), self will remain empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut v = vec![1, 2, 3];
    /// let u: Vec<_> = v.par_drain(1..).num_threads(2).collect();
    ///
    /// assert_eq!(v, &[1]);
    /// assert_eq!(u, &[2, 3]);
    /// ```
    fn par_drain<R>(
        &mut self,
        range: R,
    ) -> Par<<Self as ConcurrentDrainableOverSlice>::DrainingIter<'_>, DefaultOrchestrator>
    where
        R: RangeBounds<usize>,
    {
        Par::new(Params::default(), self.con_drain(range))
    }
}

impl<I> ParallelDrainableOverSlice for I where I: ConcurrentDrainableOverSlice {}
