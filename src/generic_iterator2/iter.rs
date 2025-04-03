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
}
