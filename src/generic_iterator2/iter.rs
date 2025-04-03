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
