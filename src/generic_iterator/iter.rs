use crate::ParIter;

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

impl<T, S> GenericIterator<T, S, rayon::iter::Empty<T>, crate::iter::ParEmpty<T>>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
{
    pub fn sequential(iter: S) -> Self {
        Self::Sequential(iter)
    }
}

impl<T, R> GenericIterator<T, core::iter::Empty<T>, R, crate::iter::ParEmpty<T>>
where
    T: Send + Sync,
    R: rayon::iter::ParallelIterator<Item = T>,
{
    pub fn rayon(iter: R) -> Self {
        Self::Rayon(iter)
    }
}

impl<T, O> GenericIterator<T, core::iter::Empty<T>, rayon::iter::Empty<T>, O>
where
    T: Send + Sync,
    O: ParIter<Item = T>,
{
    pub fn orx(iter: O) -> Self {
        Self::Orx(iter)
    }
}
