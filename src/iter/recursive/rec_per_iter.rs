use crate::{ParallelRunner, computational_variants::Par};
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter, implementations::ConIterVec};
use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;

impl<E, I, R> Par<ConcurrentRecursiveIter<I::Item, E, I>, R>
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Send,
    E: Fn(&I::Item) -> I + Sync,
    R: ParallelRunner,
{
    pub fn into_eager(self) -> Par<ConIterVec<I::Item>, R> {
        let (orchestrator, params, iter) = self.destruct();
        let items: Vec<_> = iter.into_seq_iter().collect();
        let iter = items.into_con_iter();
        Par::new(orchestrator, params, iter)
    }
}
