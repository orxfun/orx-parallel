use crate::{ParIter, ParallelRunner};

/// A parallel iterator with a known and fixed size,
/// meaning all transformations are 1 to 1.
pub trait ParEnumerate<R: ParallelRunner>: ParIter<R> {
    /// Creates an iterator which gives each value along with its index in the source collection.
    ///
    /// The iterator returned yields pairs `(i, val)`, where `i` is the index in the source collection,
    /// and `val` is the value returned by the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_parallel::*;
    ///
    /// let vec = vec![26i32, -27, 5];
    ///
    /// let max_abs = vec.into_par().enumerate().max_by_key(|(_idx, x)| x.abs());
    /// assert_eq!(max_abs, Some((1, -27)));
    /// ```
    fn enumerate(self) -> impl ParIter<R, Item = (usize, Self::Item)>;
}
