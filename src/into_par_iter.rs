use crate::{computational_variants::Par, runner::DefaultRunner, ParIter, Params};
use orx_concurrent_iter::IntoConcurrentIter;

pub trait IntoParIter: IntoConcurrentIter {
    fn into_par(self) -> impl ParIter<DefaultRunner, Item = Self::Item>;
}

impl<I> IntoParIter for I
where
    I: IntoConcurrentIter,
{
    fn into_par(self) -> impl ParIter<DefaultRunner, Item = Self::Item> {
        Par::new(Params::default(), self.into_con_iter())
    }
}
