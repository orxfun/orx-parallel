#[cfg(test)]
use crate::IterationOrder;
use crate::computations::{UM, Using};
#[cfg(test)]
use crate::runner::parallel_runner_compute::u_collect_arbitrary;
use crate::runner::parallel_runner_compute::u_collect_ordered;
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

impl<U, I, O, M1> UM<U, I, O, M1>
where
    U: Using,
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<O>,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        match (p.is_sequential(), p.iteration_order) {
            (true, _) => (0, self.sequential(pinned_vec)),
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
        let (mut using, _, iter, map1) = self.destruct();
        let mut u = using.create(0);

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(&mut u, i));
        }

        pinned_vec
    }
}
