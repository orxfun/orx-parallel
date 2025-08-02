use crate::computations::{UX, Using, Values};
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
{
    pub fn next(self) -> Option<Vo::Item> {
        let (using, _, iter, xap1) = self.destruct();
        let mut u = using.into_inner();
        iter.next()
            .and_then(|i| xap1(&mut u, i).values().into_iter().next())
    }
}
