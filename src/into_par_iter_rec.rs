use crate::{DefaultRunner, Params, computational_variants::Par};
use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;

pub trait IntoParIterRec
where
    Self: IntoIterator,
    Self::Item: Send,
{
    fn into_par_rec<E, I>(
        self,
        extend: E,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, E, I>, DefaultRunner>
    where
        I: IntoIterator<Item = Self::Item>,
        I::IntoIter: ExactSizeIterator,
        E: Fn(&Self::Item) -> I + Sync;
}

impl<X> IntoParIterRec for X
where
    X: IntoIterator,
    X::Item: Send,
{
    fn into_par_rec<E, I>(
        self,
        extend: E,
    ) -> Par<ConcurrentRecursiveIter<<Self as IntoIterator>::Item, E, I>, DefaultRunner>
    where
        I: IntoIterator<Item = <Self as IntoIterator>::Item>,
        I::IntoIter: ExactSizeIterator,
        E: Fn(&<Self as IntoIterator>::Item) -> I + Sync,
    {
        let con_rec_iter = ConcurrentRecursiveIter::new(extend, self);
        Par::new(DefaultRunner::default(), Params::default(), con_rec_iter)
    }
}
