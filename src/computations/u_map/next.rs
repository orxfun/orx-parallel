use super::m::UM;
use crate::computations::Using;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, O, M1> UM<U, I, O, M1>
where
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync,
{
    pub fn next(self) -> Option<O> {
        let (mut using, _, iter, map1) = self.destruct();
        let mut u = using.create(0);
        iter.next().map(|x| map1(&mut u, x))
    }
}
