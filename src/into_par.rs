use crate::{par_iter::ParEmpty, parameters::Params};
use orx_concurrent_iter::{
    implementations::ConIterOfIter, ConcurrentIter, IntoConcurrentIter, IterIntoConcurrentIter,
    Regular,
};

pub trait IntoPar: IntoConcurrentIter {
    type ParItem: Send + Sync;

    type ConIntoIter: ConcurrentIter<Item = Self::ParItem>;

    fn into_par(self) -> ParEmpty<Self::ConIntoIter>;
}

impl<I: IntoConcurrentIter> IntoPar for I {
    type ParItem = I::Item;

    type ConIntoIter = I::IntoIter;

    fn into_par(self) -> ParEmpty<Self::ConIntoIter> {
        ParEmpty::new(self.into_concurrent_iter(), Params::default())
    }
}

pub trait IteratorIntoPar: IterIntoConcurrentIter
where
    Self: Sized + Iterator,
    Self::Item: Send + Sync,
{
    type ParItem: Send + Sync;

    fn iter_into_par(self) -> ParEmpty<ConIterOfIter<Self, Regular>>;
}

impl<I> IteratorIntoPar for I
where
    I: Sized + Iterator,
    I::Item: Send + Sync,
{
    type ParItem = Self::Item;

    fn iter_into_par(self) -> ParEmpty<ConIterOfIter<Self, Regular>> {
        ParEmpty::new(self.iter_into_concurrent_iter(), Params::default())
    }
}
