use super::m::M;
use orx_concurrent_iter::ConcurrentIter;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
    pub fn next(self) -> Option<O> {
        let (_, iter, map1) = self.destruct();
        iter.next().map(map1)
    }
}
