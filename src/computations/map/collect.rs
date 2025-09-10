use super::m::M;
#[cfg(test)]
use crate::IterationOrder;
use crate::orch::Orchestrator;
#[cfg(test)]
use crate::runner::parallel_runner_compute::collect_arbitrary;
use crate::runner::parallel_runner_compute::collect_ordered;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

impl<R, I, O, M1> M<R, I, O, M1>
where
    R: Orchestrator,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(I::Item) -> O + Sync,
{
    pub fn collect_into<P>(self, pinned_vec: P) -> (usize, P)
    where
        P: IntoConcurrentPinnedVec<O>,
    {
        // let (_, p) = self.len_and_params();
        // match (p.is_sequential(), p.iteration_order) {
        //     (true, _) => (0, self.sequential(pinned_vec)),
        //     #[cfg(test)]
        //     (false, IterationOrder::Arbitrary) => collect_arbitrary::m(self, pinned_vec),
        //     (false, _) => collect_ordered::m(self, pinned_vec),
        // }
        todo!()
    }

    fn sequential<P>(self, mut pinned_vec: P) -> P
    where
        P: IntoConcurrentPinnedVec<O>,
    {
        let (_, _, iter, map1) = self.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(i));
        }

        pinned_vec
    }
}
