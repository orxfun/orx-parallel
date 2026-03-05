use super::iter::GenericIterator;
use crate::ParIter;
use core::cmp::Ordering;

impl<T, S, R, O> GenericIterator<T, S, R, O>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
    /// Reduction for the generic iterator.
    ///
    /// See [`reduce`] for details.
    ///
    /// [`reduce`]: crate::ParIter::reduce
    pub fn reduce<Reduce>(self, reduce: Reduce) -> Option<T>
    where
        Reduce: Fn(T, T) -> T + Send + Sync,
    {
        match self {
            GenericIterator::Sequential(x) => x.reduce(reduce),
            GenericIterator::Rayon(x) => x.reduce_with(reduce),
            GenericIterator::Orx(x) => x.reduce(reduce),
        }
    }

    /// Count reduction for the generic iterator.
    ///
    /// See [`count`] for details.
    ///
    /// [`count`]: crate::ParIter::count
    pub fn count(self) -> usize {
        match self {
            GenericIterator::Sequential(x) => x.count(),
            GenericIterator::Rayon(x) => x.count(),
            GenericIterator::Orx(x) => x.count(),
        }
    }

    /// For-each iteration for the generic iterator.
    ///
    /// See [`for_each`] for details.
    ///
    /// [`for_each`]: crate::ParIter::for_each
    pub fn for_each<Operation>(self, operation: Operation)
    where
        Operation: Fn(T) + Sync + Send,
    {
        match self {
            GenericIterator::Sequential(x) => x.for_each(operation),
            GenericIterator::Rayon(x) => x.for_each(operation),
            GenericIterator::Orx(x) => x.for_each(operation),
        }
    }

    /// Max reduction for the generic iterator.
    ///
    /// See [`max`] for details.
    ///
    /// [`max`]: crate::ParIter::max
    pub fn max(self) -> Option<T>
    where
        T: Ord,
    {
        match self {
            GenericIterator::Sequential(x) => x.max(),
            GenericIterator::Rayon(x) => x.max(),
            GenericIterator::Orx(x) => x.max(),
        }
    }

    /// Max-by reduction for the generic iterator.
    ///
    /// See [`max_by`] for details.
    ///
    /// [`max_by`]: crate::ParIter::max_by
    pub fn max_by<Compare>(self, compare: Compare) -> Option<T>
    where
        Compare: Fn(&T, &T) -> Ordering + Sync + Send,
    {
        match self {
            GenericIterator::Sequential(x) => x.max_by(compare),
            GenericIterator::Rayon(x) => x.max_by(compare),
            GenericIterator::Orx(x) => x.max_by(compare),
        }
    }

    /// Max-by-key reduction for the generic iterator.
    ///
    /// See [`max_by_key`] for details.
    ///
    /// [`max_by_key`]: crate::ParIter::max_by_key
    pub fn max_by_key<Key, GetKey>(self, key: GetKey) -> Option<T>
    where
        Key: Ord + Send,
        GetKey: Fn(&T) -> Key + Sync + Send,
    {
        match self {
            GenericIterator::Sequential(x) => x.max_by_key(key),
            GenericIterator::Rayon(x) => x.max_by_key(key),
            GenericIterator::Orx(x) => x.max_by_key(key),
        }
    }

    /// Min reduction for the generic iterator.
    ///
    /// See [`min`] for details.
    ///
    /// [`min`]: crate::ParIter::min
    pub fn min(self) -> Option<T>
    where
        T: Ord,
    {
        match self {
            GenericIterator::Sequential(x) => x.min(),
            GenericIterator::Rayon(x) => x.min(),
            GenericIterator::Orx(x) => x.min(),
        }
    }

    /// Min-by reduction for the generic iterator.
    ///
    /// See [`min_by`] for details.
    ///
    /// [`min_by`]: crate::ParIter::min_by
    pub fn min_by<Compare>(self, compare: Compare) -> Option<T>
    where
        Compare: Fn(&T, &T) -> Ordering + Sync + Send,
    {
        match self {
            GenericIterator::Sequential(x) => x.min_by(compare),
            GenericIterator::Rayon(x) => x.min_by(compare),
            GenericIterator::Orx(x) => x.min_by(compare),
        }
    }

    /// Min-by-key reduction for the generic iterator.
    ///
    /// See [`min_by_key`] for details.
    ///
    /// [`min_by_key`]: crate::ParIter::min_by_key
    pub fn min_by_key<Key, GetKey>(self, key: GetKey) -> Option<T>
    where
        Key: Ord + Send,
        GetKey: Fn(&T) -> Key + Sync + Send,
    {
        match self {
            GenericIterator::Sequential(x) => x.min_by_key(key),
            GenericIterator::Rayon(x) => x.min_by_key(key),
            GenericIterator::Orx(x) => x.min_by_key(key),
        }
    }

    /// Sum reduction for the generic iterator.
    ///
    /// See [`sum`] for details.
    ///
    /// [`sum`]: crate::ParIter::sum
    pub fn sum(self) -> T
    where
        T: crate::special_type_sets::Sum<T> + core::iter::Sum<T>,
    {
        match self {
            GenericIterator::Sequential(x) => x.sum(),
            GenericIterator::Rayon(x) => x.sum(),
            GenericIterator::Orx(x) => x.sum(),
        }
    }
}
