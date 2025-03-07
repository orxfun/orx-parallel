use crate::{computational_variants::Par, runner::DefaultRunner, ParIter, Params};
use orx_concurrent_iter::IterIntoConcurrentIter;

pub trait IterIntoParIter: Iterator {
    fn iter_into_par(self) -> impl ParIter<DefaultRunner, Item = Self::Item>;
}

impl<I> IterIntoParIter for I
where
    I: Iterator,
    I::Item: Send + Sync,
{
    fn iter_into_par(self) -> impl ParIter<DefaultRunner, Item = Self::Item> {
        Par::new(Params::default(), self.iter_into_con_iter())
    }
}
