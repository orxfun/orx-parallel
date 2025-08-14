use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using::Using;
use crate::using::computations::UX;
use crate::using::runner::parallel_runner_compute::{u_next, u_next_any};
use crate::values::TransformableValues;
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: TransformableValues,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    Vo::Item: Send,
{
    pub fn next2(self) -> Option<Vo::Item> {
        let (using, _, iter, xap1) = self.destruct();
        let mut u = using.into_inner();
        iter.next()
            .and_then(|i| xap1(&mut u, i).values().into_iter().next())
    }

    pub fn next<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        u_next::u_x(R::early_return(p, len), self)
    }

    pub fn next_any<R>(self) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        u_next_any::u_x(R::early_return(p, len), self)
    }
}
