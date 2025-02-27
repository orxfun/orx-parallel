use crate::{par_iterators::Par, parameters::Params};
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterable};

pub trait Parallelizable: ConcurrentIterable {
    type ParItem;

    type ConIter: ConcurrentIter<Item = Self::ParItem>;

    fn par(&self) -> Par<Self::ConIter>;
}

impl<I> Parallelizable for I
where
    I: ConcurrentIterable,
{
    type ParItem = I::Item;

    type ConIter = I::Iter;

    fn par(&self) -> Par<Self::ConIter> {
        Par::new(self.concurrent_iter(), Params::default())
    }
}
