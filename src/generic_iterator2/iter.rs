use crate::{IntoParIter, ParIter};
use rayon::iter::IntoParallelIterator;

pub enum GenericIterator<T, S, R, O>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
    Sequential(S),
    Rayon(R),
    Orx(O),
}

impl<T, S, R, O> GenericIterator<T, S, R, O>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
    // constructors

    pub fn sequential(iter: S) -> Self {
        Self::Sequential(iter)
    }

    pub fn rayon(iter: R) -> Self {
        Self::Rayon(iter)
    }

    pub fn orx(iter: O) -> Self {
        Self::Orx(iter)
    }

    // computation transformations

    pub fn map<Out, Map>(
        self,
        map: Map,
    ) -> GenericIterator<
        Out,
        impl Iterator<Item = Out>,
        impl rayon::iter::ParallelIterator<Item = Out>,
        impl ParIter<Item = Out>,
    >
    where
        Out: Send + Sync,
        Map: Fn(T) -> Out + Send + Sync + Clone,
    {
        match self {
            GenericIterator::Sequential(x) => GenericIterator::Sequential(x.map(map)),
            GenericIterator::Rayon(x) => GenericIterator::Rayon(x.map(map)),
            GenericIterator::Orx(x) => GenericIterator::Orx(x.map(map)),
        }
    }
}

pub fn any_sequential<T>() -> impl Iterator<Item = T> {
    vec![].into_iter()
}

pub fn any_rayon<T: Send + Sync>() -> impl rayon::iter::ParallelIterator<Item = T> {
    vec![].into_par_iter()
}

pub fn any_orx<T: Send + Sync>() -> impl ParIter<Item = T> {
    vec![].into_par()
}
