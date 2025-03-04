use crate::{
    computational_variants::Par,
    runner::{DefaultRunner, ParallelRunner},
    ParIter, Params,
};
use orx_concurrent_iter::IterIntoConcurrentIter;

pub trait IterIntoParIter<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Item: Send + Sync;

    fn iter_into_par(self) -> impl ParIter<R, Item = Self::Item>;
}

impl<I, R> IterIntoParIter<R> for I
where
    I: IterIntoConcurrentIter,
    R: ParallelRunner,
{
    type Item = I::Item;

    fn iter_into_par(self) -> impl ParIter<R, Item = Self::Item> {
        Par::new(Params::default(), self.iter_into_con_iter())
    }
}
