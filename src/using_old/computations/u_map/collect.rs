use super::m::UM;
#[cfg(test)]
use crate::IterationOrder;
use crate::orch::NumSpawned;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use crate::using_old::Using;
#[cfg(test)]
use crate::using_old::runner::parallel_runner_compute::u_collect_arbitrary;
use crate::using_old::runner::parallel_runner_compute::u_collect_ordered;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

impl<U, I, O, M1> UM<U, I, O, M1>
where
    U: Using,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (NumSpawned, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<O>,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        match (p.is_sequential(), p.iteration_order) {
            (true, _) => (NumSpawned::zero(), self.sequential(pinned_vec)),
            #[cfg(test)]
            (false, IterationOrder::Arbitrary) => {
                u_collect_arbitrary::u_m(R::collection(p, len), self, pinned_vec)
            }
            (false, _) => u_collect_ordered::u_m(R::collection(p, len), self, pinned_vec),
        }
    }

    fn sequential<P>(self, mut pinned_vec: P) -> P
    where
        P: IntoConcurrentPinnedVec<O>,
    {
        let (using, _, iter, map1) = self.destruct();
        let mut u = using.into_inner();

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(&mut u, i));
        }

        pinned_vec
    }
}
