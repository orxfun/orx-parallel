use crate::infallible::{Par, xap_variants::Id};
use crate::runner::default_runner;
use orx_concurrent_iter::IntoConcurrentIter;
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

pub trait IntoRecParIter
where
    Self: IntoIterator,
    Self::Item: Send,
{
    fn into_rec_par<F>(
        self,
        extend: F,
        exact_len: Option<usize>,
    ) -> Par<ConcurrentRecursiveIter<Self::Item, F>, Id<Self::Item>>
    where
        F: Fn(&Self::Item, &Queue<Self::Item>) + Sync;
}
