use super::x::X;
use crate::runner::parallel_runner_compute::{collect_arbitrary, collect_ordered};
use crate::values::runner_results::Fallability;
use crate::{
    IterationOrder,
    runner::{ParallelRunner, ParallelRunnerCompute},
    values::Values,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let (len, p) = self.len_and_params();
        match (p.is_sequential(), p.iteration_order) {
            (true, _) => (0, self.sequential(pinned_vec)),
            (false, IterationOrder::Arbitrary) => {
                let (num_threads, result) =
                    collect_arbitrary::x(R::collection(p, len), self, pinned_vec);
                // TODO: Values abstraction might be revisited to represent that this infallible
                let pinned_vec = result.to_collected();
                (num_threads, pinned_vec)
            }
            (false, IterationOrder::Ordered) => {
                let (num_threads, result) =
                    collect_ordered::x(R::collection(p, len), self, pinned_vec);
                // TODO: Values abstraction might be revisited to represent that this infallible
                let pinned_vec = result.to_collected();
                (num_threads, pinned_vec)
            }
        }
    }

    pub fn try_collect_into<R, P>(
        self,
        pinned_vec: P,
    ) -> (usize, Result<P, <Vo::Fallability as Fallability>::Error>)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let (len, p) = self.len_and_params();
        match (p.is_sequential(), p.iteration_order) {
            (true, _) => (0, Ok(self.sequential(pinned_vec))),
            (false, IterationOrder::Arbitrary) => {
                let (nt, result) = collect_arbitrary::x(R::collection(p, len), self, pinned_vec);
                (nt, result.to_result())
            }
            (false, IterationOrder::Ordered) => {
                let (nt, result) = collect_ordered::x(R::collection(p, len), self, pinned_vec);
                (nt, result.to_result())
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
            let stop = vt.push_to_pinned_vec(&mut pinned_vec);
            if stop {
                break;
            }
        }

        pinned_vec
    }
}
