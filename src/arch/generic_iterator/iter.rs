use crate::ParIter;

/// An iterator that generalizes over:
///
/// * sequential iterators,
/// * rayon's parallel iterators, and
/// * orx-parallel's parallel iterators.
///
/// This is particularly useful for enabling a convenient way to run experiments
/// using these different computation approaches.
pub enum GenericIterator<T, S, R, O>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
    R: rayon::iter::ParallelIterator<Item = T>,
    O: ParIter<Item = T>,
{
    /// Sequential, or regular, iterator.
    Sequential(S),
    /// rayon's parallel iterator.
    Rayon(R),
    /// orx-parallel's parallel iterator.
    Orx(O),
}

impl<T, S> GenericIterator<T, S, rayon::iter::Empty<T>, crate::iter::ParEmpty<T>>
where
    T: Send + Sync,
    S: Iterator<Item = T>,
{
    /// Creates the generic iterator from sequential iterator variant.
    pub fn sequential(iter: S) -> Self {
        Self::Sequential(iter)
    }
}

impl<T, R> GenericIterator<T, core::iter::Empty<T>, R, crate::iter::ParEmpty<T>>
where
    T: Send + Sync,
    R: rayon::iter::ParallelIterator<Item = T>,
{
    /// Creates the generic iterator from rayon iterator variant.
    pub fn rayon(iter: R) -> Self {
        Self::Rayon(iter)
    }
}

impl<T, O> GenericIterator<T, core::iter::Empty<T>, rayon::iter::Empty<T>, O>
where
    T: Send + Sync,
    O: ParIter<Item = T>,
{
    /// Creates the generic iterator from orx-parallel iterator variant.
    pub fn orx(iter: O) -> Self {
        Self::Orx(iter)
    }
}
