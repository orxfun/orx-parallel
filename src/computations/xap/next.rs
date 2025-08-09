use super::x::X;
use crate::computations::Values;
use orx_concurrent_iter::ConcurrentIter;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values,
    M1: Fn(I::Item) -> Vo,
{
    pub fn next(self) -> Option<Vo::Item> {
        // TODO: to be parallelized!
        let (_, iter, xap1) = self.destruct();
        while let Some(i) = iter.next() {
            let next = xap1(i).values().into_iter().next();
            if next.is_some() {
                return next;
            }
        }
        None
    }
}
