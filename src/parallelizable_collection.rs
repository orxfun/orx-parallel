use crate::{par_iter::ParEmpty, parallelizable::Parallelizable};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParallelizableCollection {
    type ParItem;

    type Parallelizable<'i>: Parallelizable<ParItem = &'i Self::ParItem>
    where
        Self: 'i;

    fn as_parallelizable(&self) -> Self::Parallelizable<'_>;

    fn par(&self) -> ParEmpty<<Self::Parallelizable<'_> as Parallelizable>::ParIter> {
        self.as_parallelizable().par()
    }
}
