use crate::{par_iter::ParEmpty, parameters::Params};
use orx_concurrent_iter::{
    implementations::ConIterOfIter, ConcurrentIter, IntoConcurrentIter, IterIntoConcurrentIter,
    Regular,
};

pub trait IntoPar {
    type Item: Send + Sync;

    type IntoIter: ConcurrentIter<Item = Self::Item>;

    fn into_par(self) -> ParEmpty<Self::IntoIter>;
}

impl<I: IntoConcurrentIter> IntoPar for I {
    type Item = I::Item;

    type IntoIter = I::IntoIter;

    fn into_par(self) -> ParEmpty<Self::IntoIter> {
        ParEmpty::new(self.into_concurrent_iter(), Params::default())
    }
}

pub trait IteratorIntoPar
where
    Self: Sized + Iterator,
    Self::Item: Send + Sync,
{
    fn iter_into_par(self) -> ParEmpty<ConIterOfIter<Self, Regular>>;
}

impl<I> IteratorIntoPar for I
where
    I: Sized + Iterator,
    I::Item: Send + Sync,
{
    fn iter_into_par(self) -> ParEmpty<ConIterOfIter<Self, Regular>> {
        ParEmpty::new(self.iter_into_concurrent_iter(), Params::default())
    }
}
