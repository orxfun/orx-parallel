use super::x::X;
use crate::generic_values::runner_results::{
    Fallibility, Infallible, ParallelCollect, ParallelCollectArbitrary, Stop,
};
use crate::runner::parallel_runner_compute::{collect_arbitrary, collect_ordered};
use crate::{
    IterationOrder,
    generic_values::Values,
    runner::{ParallelRunner, ParallelRunnerCompute},
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
    pub fn try_collect_into<R, P>(
        self,
        pinned_vec: P,
    ) -> (usize, Result<P, <Vo::Fallibility as Fallibility>::Error>)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        todo!()
        // let (len, p) = self.len_and_params();
        // match (p.is_sequential(), p.iteration_order) {
        //     (true, _) => (0, self.try_sequential(pinned_vec)),
        //     (false, IterationOrder::Arbitrary) => {
        //         let (nt, result) = collect_arbitrary::x(R::collection(p, len), self, pinned_vec);
        //         (nt, result.into_result())
        //     }
        //     (false, IterationOrder::Ordered) => {
        //         let (nt, result) = collect_ordered::x(R::collection(p, len), self, pinned_vec);
        //         (nt, result.into_result())
        //     }
        // }
    }

    fn try_sequential<P>(
        self,
        mut pinned_vec: P,
    ) -> Result<P, <Vo::Fallibility as Fallibility>::Error>
    where
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let (_, iter, xap1) = self.destruct();

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = xap1(i);
            let done = vt.push_to_pinned_vec(&mut pinned_vec);
            if let Some(stop) = Vo::sequential_push_to_stop(done) {
                match stop {
                    Stop::DueToWhile => return Ok(pinned_vec),
                    Stop::DueToError { error } => return Err(error),
                }
            }
        }

        Ok(pinned_vec)
    }
}
