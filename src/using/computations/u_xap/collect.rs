use crate::using::Using;
use crate::using::computations::UX;
use crate::using::runner::parallel_runner_compute::{u_collect_arbitrary, u_collect_ordered};
use crate::{
    IterationOrder,
    computations::Values,
    runner::{ParallelRunner, ParallelRunnerCompute},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
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
                u_collect_arbitrary::u_x(R::collection(p, len), self, pinned_vec)
            }
            (false, IterationOrder::Ordered) => {
                u_collect_ordered::u_x(R::collection(p, len), self, pinned_vec)
            }
        }
    }

    fn sequential<P>(self, mut pinned_vec: P) -> P
    where
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let (using, _, iter, xap1) = self.destruct();
        let mut u = using.into_inner();

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = xap1(&mut u, i);
            vt.push_to_pinned_vec(&mut pinned_vec);
        }

        pinned_vec
    }
}
