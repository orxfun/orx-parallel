use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use core::ops::RangeBounds;
use orx_concurrent_iter::ConcurrentDrainableOverSlice;

/// Adds parallel draining to slice-based drainable collections.
///
/// Sequential counterpart: draining methods such as `Vec::drain`.
pub trait ParDrain: ConcurrentDrainableOverSlice {
    /// Drains the specified range and returns a parallel iterator over removed items.
    ///
    /// # Example
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let mut values = vec![0i32, 1, 2, 3, 4, 5];
    /// let drained_sum: i32 = ParDrain::par_drain(&mut values, 0..3).sum();
    ///
    /// assert_eq!(drained_sum, 3);
    /// assert_eq!(values, vec![3, 4, 5]);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `range` is invalid for the underlying collection.
    fn par_drain<R>(
        &mut self,
        range: R,
    ) -> ParIter<<Self as ConcurrentDrainableOverSlice>::DrainingIter<'_>, Id<Self::Item>>
    where
        R: RangeBounds<usize>,
    {
        ParIter::new(
            self.con_drain(range),
            Id::new(),
            default_runner(),
            Default::default(),
        )
    }
}

impl<I> ParDrain for I where I: ConcurrentDrainableOverSlice {}
