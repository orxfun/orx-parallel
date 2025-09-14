use crate::generic_values::runner_results::{
    Infallible, ParallelCollect, ParallelCollectArbitrary,
};
use crate::orch::NumSpawned;
use crate::using::Using;
use crate::using::computations::UX;
use crate::using::runner::parallel_runner_compute::{u_collect_arbitrary, u_collect_ordered};
use crate::{
    IterationOrder,
    generic_values::Values,
    runner::{ParallelRunner, ParallelRunnerCompute},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

impl<U, I, Vo, M1> UX<U, I, Vo, M1>
where
    U: Using,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (NumSpawned, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
        Vo: Values<Fallibility = Infallible>,
    {
        let (len, p) = self.len_and_params();

        match (p.is_sequential(), p.iteration_order) {
            (true, _) => (NumSpawned::zero(), self.sequential(pinned_vec)),
            (false, IterationOrder::Arbitrary) => {
                let (num_threads, result) =
                    u_collect_arbitrary::u_x(R::collection(p, len), self, pinned_vec);
                let pinned_vec = match result {
                    ParallelCollectArbitrary::AllOrUntilWhileCollected { pinned_vec } => pinned_vec,
                };
                (num_threads, pinned_vec)
            }
            (false, IterationOrder::Ordered) => {
                let (num_threads, result) =
                    u_collect_ordered::u_x(R::collection(p, len), self, pinned_vec);
                let pinned_vec = match result {
                    ParallelCollect::AllCollected { pinned_vec } => pinned_vec,
                    ParallelCollect::StoppedByWhileCondition {
                        pinned_vec,
                        stopped_idx: _,
                    } => pinned_vec,
                };
                (num_threads, pinned_vec)
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
            let done = vt.push_to_pinned_vec(&mut pinned_vec);
            if Vo::sequential_push_to_stop(done).is_some() {
                break;
            }
        }

        pinned_vec
    }
}
