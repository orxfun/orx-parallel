use crate::infallible::{Par, xap_variants::Id};
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
        exact_len: Option<usize>,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, F>, Id<Self::Item>>
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
        exact_len: Option<usize>,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, F>, Id<Self::Item>>
    where
        F: Fn(&Self::Item, &Queue<Self::Item>) + Sync,
    {
        let iter = match exact_len {
            Some(exact_len) => ConcurrentRecursiveIter::new_exact(self, extend, exact_len),
            None => ConcurrentRecursiveIter::new(self, extend),
        };
        Par::new(iter, Id::new(), default_runner(), Default::default())
    }
}
