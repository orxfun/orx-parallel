use crate::{par_iter::ParEmpty, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterable};

pub trait Parallelizable: ConcurrentIterable {
    type ParItem;

    type ConIter: ConcurrentIter<Item = Self::ParItem>;

    fn par(&self) -> ParEmpty<Self::ConIter>;
}

impl<I> Parallelizable for I
where
    I: ConcurrentIterable,
{
    type ParItem = I::Item;

    type ConIter = I::Iter;

    fn par(&self) -> ParEmpty<Self::ConIter> {
        ParEmpty::new(self.concurrent_iter(), Params::default())
    }
}
