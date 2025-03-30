use super::m::M;
use crate::runner::ParallelRunner;
use orx_concurrent_iter::ConcurrentIter;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
{
    pub fn first<R>(self) -> Option<O>
    where
        R: ParallelRunner,
    {
        let (_, iter, map1) = self.destruct();
        iter.next().map(map1)
    }
}
