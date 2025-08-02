use crate::IterationOrder;
use crate::computations::{UXfx, Using, Values};
use crate::runner::parallel_runner_compute::{u_collect_arbitrary, u_collect_ordered};
use crate::runner::{ParallelRunner, ParallelRunnerCompute};
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

impl<U, I, Vt, Vo, M1, F, M2> UXfx<U, I, Vt, Vo, M1, F, M2>
where
    U: Using,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(&mut U::Item, I::Item) -> Vt + Send + Sync,
    F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
    M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync,
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
                u_collect_arbitrary::u_xfx(R::collection(p, len), self, pinned_vec)
            }
            (false, IterationOrder::Ordered) => {
                u_collect_ordered::u_xfx(R::collection(p, len), self, pinned_vec)
            }
        }
    }

    fn sequential<P>(self, mut pinned_vec: P) -> P
    where
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        let (using, _, iter, xap1, filter, xap2) = self.destruct();
        let mut u = using.into_inner();

        let iter = iter.into_seq_iter();
        for i in iter {
            let vt = xap1(&mut u, i);
            vt.u_filter_map_collect_sequential(&mut u, &filter, &xap2, &mut pinned_vec);
        }

        pinned_vec
    }
}
