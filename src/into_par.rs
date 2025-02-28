use crate::{
    computations::{DefaultRunner, ParallelRunner},
    par_iterators::Par,
    parameters::Params,
    ParIter,
};
use orx_concurrent_iter::{
    implementations::ConIterOfIter, IntoConcurrentIter, IterIntoConcurrentIter,
};

pub trait IntoPar<R = DefaultRunner>
where
    R: ParallelRunner,
{
    type ParItem: Send + Sync;

    fn into_par(self) -> impl ParIter<R, Item = Self::ParItem>;
}

impl<I: IntoConcurrentIter> IntoPar for I {
    type ParItem = I::Item;

    fn into_par(self) -> impl ParIter<Item = Self::ParItem> {
        Par::new(self.into_concurrent_iter(), Params::default())
    }
}

pub trait IteratorIntoPar
where
    Self: Sized + Iterator,
    Self::Item: Send + Sync,
{
    type ParItem: Send + Sync;

    fn iter_into_par(self) -> Par<ConIterOfIter<Self>>;
}

impl<I> IteratorIntoPar for I
where
    I: Sized + Iterator,
    I::Item: Send + Sync,
{
    type ParItem = Self::Item;

    fn iter_into_par(self) -> Par<ConIterOfIter<Self>> {
        Par::new(self.iter_into_concurrent_iter(), Params::default())
    }
}
