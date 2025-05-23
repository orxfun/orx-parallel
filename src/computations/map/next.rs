use super::m::M;
use orx_concurrent_iter::ConcurrentIter;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
{
    pub fn next(self) -> Option<O> {
        let (_, iter, map1) = self.destruct();
        iter.next().map(map1)
    }
}
