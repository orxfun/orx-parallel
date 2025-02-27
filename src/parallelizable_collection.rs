use crate::{into_par::IntoPar, par_iterators::Par, parallelizable::Parallelizable};
use orx_concurrent_iter::IntoConcurrentIter;

pub trait ParallelizableCollection {
    type ParItem;

    type Parallelizable<'i>: Parallelizable<ParItem = &'i Self::ParItem>
    where
        Self: 'i;

    fn as_parallelizable(&self) -> Self::Parallelizable<'_>;

    fn par(&self) -> Par<<Self::Parallelizable<'_> as Parallelizable>::ConIter> {
        self.as_parallelizable().par()
    }
}

impl<X> ParallelizableCollection for X
where
    X: IntoConcurrentIter,
    for<'a> &'a X: IntoConcurrentIter<Item = &'a <X as IntoConcurrentIter>::Item>,
{
    type ParItem = <X as IntoPar>::ParItem;

    type Parallelizable<'i>
        = &'i X
    where
        Self: 'i;

    fn as_parallelizable(&self) -> Self::Parallelizable<'_> {
        self
    }
}
