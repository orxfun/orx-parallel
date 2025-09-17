use crate::Params;
use crate::collect_into::ParCollectIntoCore;
use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::orch::Orchestrator;
use crate::using::using_variants::Using;
use orx_concurrent_iter::ConcurrentIter;

pub trait UParCollectIntoCore<O>: ParCollectIntoCore<O> {
    fn u_m_collect_into<U, R, I, M1>(
        self,
        using: U,
        orchestrator: R,
        params: Params,
        iter: I,
        map1: M1,
    ) -> Self
    where
        U: Using,
        R: Orchestrator,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Sync;

    fn u_x_collect_into<U, R, I, Vo, X1>(
        self,
        using: U,
        orchestrator: R,
        params: Params,
        iter: I,
        xap1: X1,
    ) -> Self
    where
        U: Using,
        R: Orchestrator,
        I: ConcurrentIter,
        Vo: TransformableValues<Item = O, Fallibility = Infallible>,
        X1: Fn(&mut U::Item, I::Item) -> Vo + Sync;
}
