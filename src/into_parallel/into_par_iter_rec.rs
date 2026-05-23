use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;

pub trait IntoParIterRecursive
where
    Self: IntoIterator,
    Self::Item: Send,
{
    fn into_par_recursive<I, F>(
        self,
        extend: F,
    ) -> ParIter<ConcurrentRecursiveIter<I, F>, Id<Self::Item>>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync;
}

impl<X> IntoParIterRecursive for X
where
    X: IntoIterator,
    X::Item: Send,
{
    fn into_par_recursive<I, F>(
        self,
        extend: F,
    ) -> ParIter<ConcurrentRecursiveIter<I, F>, Id<Self::Item>>
    where
        I: IntoIterator<Item = Self::Item>,
        F: Fn(&Self::Item) -> I + Send + Sync,
    {
        let iter = ConcurrentRecursiveIter::new(self, extend, None, None);
        ParIter::new(iter, Id::new(), default_runner(), Default::default())
    }
}
