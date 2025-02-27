use crate::{par_iter::ParEmpty, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterable};

pub trait Parallelizable {
    type ParItem;

    type ParIter: ConcurrentIter<Item = Self::ParItem>;

    fn par(&self) -> ParEmpty<Self::ParIter>;
}

impl<I> Parallelizable for I
where
    I: ConcurrentIterable,
{
    type ParItem = I::Item;

    type ParIter = I::Iter;

    fn par(&self) -> ParEmpty<Self::ParIter> {
        ParEmpty::new(self.concurrent_iter(), Params::default())
    }
}
