use super::x::X;
use crate::computations::Values;
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values + Send,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub fn next(self) -> Option<Vo::Item> {
        let (_, iter, xap1) = self.destruct();
        iter.next()
            .and_then(|i| xap1(i).values().into_iter().next())
    }
}
