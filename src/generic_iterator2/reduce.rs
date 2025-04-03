use super::iter::GenericIterator;
use crate::ParIter;

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

    // fn all<Predicate>(self, predicate: Predicate) -> bool
    // where
    //     Predicate: Fn(&T) -> bool + Send + Sync + Clone,
    // {
    //     match self {
    //         GenericIterator::Sequential(x) => x.all(predicate),
    //         GenericIterator::Rayon(x) => x.all(predicate),
    //         GenericIterator::Orx(x) => x.all(predicate),
    //     }
    // }
}
