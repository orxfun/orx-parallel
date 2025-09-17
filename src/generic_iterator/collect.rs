use super::iter::GenericIterator;
use crate::ParIter;
use alloc::vec::Vec;

impl<T, S, R, O> GenericIterator<T, S, R, O>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
    /// Collects the elements of the iterator into a vector.
    ///
    /// See [`collect`] for details of the general collect method.
    ///
    /// [`collect`]: crate::ParIter::collect
    pub fn collect_vec(self) -> Vec<T> {
        match self {
            GenericIterator::Sequential(x) => x.collect(),
            GenericIterator::Rayon(x) => x.collect(),
            GenericIterator::Orx(x) => x.collect(),
        }
    }
}
