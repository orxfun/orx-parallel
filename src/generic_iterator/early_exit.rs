use super::iter::GenericIterator;
use crate::ParIter;

impl<T, S, R, O> GenericIterator<T, S, R, O>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
    /// Find computation for the generic iterator.
    ///
    /// See [`find`] for details.
    ///
    /// [`find`]: crate::ParIter::find
    pub fn find<Predicate>(self, predicate: Predicate) -> Option<T>
    where
        Predicate: Fn(&T) -> bool + Send + Sync + Clone,
    {
        match self {
            GenericIterator::Sequential(mut x) => x.find(predicate),
            GenericIterator::Rayon(x) => x.find_first(predicate),
            GenericIterator::Orx(x) => x.find(predicate),
        }
    }

    /// Find-any computation for the generic iterator.
    ///
    /// See [`find_any`] for details.
    ///
    /// [`find_any`]: crate::ParIter::find_any
    pub fn find_any<Predicate>(self, predicate: Predicate) -> Option<T>
    where
        Predicate: Fn(&T) -> bool + Send + Sync + Clone,
    {
        match self {
            GenericIterator::Sequential(mut x) => x.find(predicate),
            GenericIterator::Rayon(x) => x.find_any(predicate),
            GenericIterator::Orx(x) => x.find_any(predicate),
        }
    }
}
