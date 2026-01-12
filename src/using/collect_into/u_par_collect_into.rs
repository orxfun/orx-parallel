use crate::Params;
use crate::collect_into::ParCollectIntoCore;
use crate::generic_values::runner_results::{Fallibility, Infallible};
use crate::generic_values::{TransformableValues, Values};
use crate::runner::ParallelRunner;
use crate::using::using_variants::Using;
use orx_concurrent_iter::ConcurrentIter;

pub trait UParCollectIntoCore<O>: ParCollectIntoCore<O> {
    fn u_m_collect_into<'using, U, R, I, M1>(
        self,
        using: U,
        orchestrator: R,
        params: Params,
        iter: I,
        map1: M1,
    ) -> Self
    where
        U: Using<'using>,
        R: ParallelRunner,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Sync;

    fn u_x_collect_into<'using, U, R, I, Vo, X1>(
        self,
        using: U,
        orchestrator: R,
        params: Params,
        iter: I,
        xap1: X1,
    ) -> Self
    where
        U: Using<'using>,
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: TransformableValues<Item = O, Fallibility = Infallible>,
        X1: Fn(*mut U::Item, I::Item) -> Vo + Sync;

    fn u_x_try_collect_into<'using, U, R, I, Vo, X1>(
        self,
        using: U,
        orchestrator: R,
        params: Params,
        iter: I,
        xap1: X1,
    ) -> Result<Self, <Vo::Fallibility as Fallibility>::Error>
    where
        U: Using<'using>,
        R: ParallelRunner,
        I: ConcurrentIter,
        X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
        Vo: Values<Item = O>,
        Self: Sized;
}
