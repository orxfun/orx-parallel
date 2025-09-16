use super::m::UM;
use crate::orch::NumSpawned;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using_old::Using;
use crate::using_old::runner::parallel_runner_compute::{u_next, u_next_any};
use orx_concurrent_iter::ConcurrentIter;

impl<U, I, O, M1> UM<U, I, O, M1>
where
    U: Using,
    I: ConcurrentIter,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    O: Send,
{
    pub fn next<R>(self) -> (NumSpawned, Option<O>)
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        u_next::u_m(R::early_return(p, len), self)
    }

    pub fn next_any<R>(self) -> (NumSpawned, Option<O>)
    where
        R: ParallelRunner,
    {
        let (len, p) = self.len_and_params();
        u_next_any::u_m(R::early_return(p, len), self)
    }
}
