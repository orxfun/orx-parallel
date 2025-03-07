use crate::{computational_variants::Par, parameters::Params, runner::DefaultRunner, ParIter};
use orx_concurrent_iter::{ConcurrentIter, ConcurrentIterable};

pub trait Parallelizable: ConcurrentIterable {
    type Item;

    type ConIter: ConcurrentIter<Item = <Self as Parallelizable>::Item>;

    fn par(&self) -> impl ParIter<DefaultRunner, Item = <Self as Parallelizable>::Item>;
}

impl<I> Parallelizable for I
where
    I: ConcurrentIterable,
{
    type Item = I::Item;

    type ConIter = I::Iter;

    fn par(&self) -> impl ParIter<DefaultRunner, Item = <Self as Parallelizable>::Item> {
        Par::new(Params::default(), self.con_iter())
    }
}
