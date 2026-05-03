use crate::infallible::{ParIter, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

pub trait IntoParIterRecursive
where
    Self: IntoIterator,
    Self::Item: Send,
{
    fn into_par_recursive<F>(
        self,
        extend: F,
    ) -> ParIter<ConcurrentRecursiveIter<Self::Item, F>, Id<Self::Item>>
    where
        F: Fn(&Self::Item, &Queue<Self::Item>) + Sync;
}

impl<X> IntoParIterRecursive for X
where
    X: IntoIterator,
    X::Item: Send,
{
    fn into_par_recursive<F>(
        self,
        extend: F,
    ) -> ParIter<ConcurrentRecursiveIter<Self::Item, F>, Id<Self::Item>>
    where
        F: Fn(&Self::Item, &Queue<Self::Item>) + Sync,
    {
        let iter = ConcurrentRecursiveIter::new(self, extend);
        ParIter::new(iter, Id::new(), default_runner(), Default::default())
    }
}
