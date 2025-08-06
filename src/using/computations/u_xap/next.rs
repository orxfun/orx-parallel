use crate::computations::Values;
use crate::using::Using;
use crate::using::computations::UX;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
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
