use super::m::M;
#[cfg(test)]
use crate::IterationOrder;
use crate::runner::parallel_runner_compute::*;
use crate::runner::{ComputationKind, ParallelRunner};
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

impl<I, O, M1> M<I, O, M1>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M1: Fn(I::Item) -> O + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<O>,
    {
        let p = self.params();
        match (p.is_sequential(), p.iteration_order) {
            (true, _) => (0, self.sequential(pinned_vec)),
            #[cfg(test)]
            (false, IterationOrder::Arbitrary) => {
                collect_arbitrary::m_collect(self.runner::<R>(), self, pinned_vec)
            }
            (false, _) => collect_ordered::m(self.runner::<R>(), self, pinned_vec),
        }
    }

    fn sequential<P>(self, mut pinned_vec: P) -> P
    where
        P: IntoConcurrentPinnedVec<O>,
    {
        let (_, iter, map1) = self.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            pinned_vec.push(map1(i));
        }

        pinned_vec
    }

    fn runner<R: ParallelRunner>(&self) -> R {
        R::new(
            ComputationKind::Collect,
            self.params(),
            self.iter().try_get_len(),
        )
    }
}
