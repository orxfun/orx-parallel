use crate::{
    computational_variants::Par,
    runner::{DefaultRunner, ParallelRunner},
    ParIter, Params,
};
use orx_concurrent_iter::IntoConcurrentIter;

pub trait IntoParIter<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type Item: Send + Sync;

    fn into_par(self) -> impl ParIter<R, Item = Self::Item>;
}

impl<I, R> IntoParIter<R> for I
where
    I: IntoConcurrentIter,
    R: ParallelRunner,
{
    type Item = I::Item;

    fn into_par(self) -> impl ParIter<R, Item = Self::Item> {
        Par::new(Params::default(), self.into_con_iter())
    }
}
