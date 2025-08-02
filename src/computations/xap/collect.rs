use super::x::X;
use crate::runner::parallel_runner_compute::*;
use crate::{
    IterationOrder,
    computations::Values,
    runner::{ParallelRunner, ParallelRunnerCompute},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let len = self.iter().try_get_len();
        let p = self.params();
        match (p.is_sequential(), p.iteration_order) {
            (true, _) => (0, self.sequential(pinned_vec)),
            (false, IterationOrder::Arbitrary) => {
                collect_arbitrary::x(R::new_collect(p, len), self, pinned_vec)
            }
            (false, IterationOrder::Ordered) => {
                collect_ordered::x(R::new_collect(p, len), self, pinned_vec)
            }
        }
    }

    fn sequential<P>(self, mut pinned_vec: P) -> P
    where
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let (_, iter, xap1) = self.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = xap1(i);
            vt.push_to_pinned_vec(&mut pinned_vec);
        }

        pinned_vec
    }
}
