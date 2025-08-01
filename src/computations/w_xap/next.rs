use super::x::WithX;
use crate::computations::Values;
use orx_concurrent_iter::ConcurrentIter;

impl<I, T, Vo, M1> WithX<I, T, Vo, M1>
where
    I: ConcurrentIter,
    T: Send + Clone,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut T, I::Item) -> Vo + Send + Sync,
{
    pub fn next(self) -> Option<Vo::Item> {
        let (_, iter, mut with, xap1) = self.destruct();
        iter.next()
            .and_then(|i| xap1(&mut with, i).values().into_iter().next())
    }
}
