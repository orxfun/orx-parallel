use super::iter::GenericIterator;
use crate::ParIter;
use std::cmp::Ordering;

impl<T, S, R, O> GenericIterator<T, S, R, O>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
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

    pub fn count(self) -> usize {
        match self {
            GenericIterator::Sequential(x) => x.count(),
            GenericIterator::Rayon(x) => x.count(),
            GenericIterator::Orx(x) => x.count(),
        }
    }

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

    pub fn sum(self) -> T
    where
        T: crate::special_type_sets::Sum<T> + std::iter::Sum<T>,
    {
        match self {
            GenericIterator::Sequential(x) => x.sum(),
            GenericIterator::Rayon(x) => x.sum(),
            GenericIterator::Orx(x) => x.sum(),
        }
    }
}
