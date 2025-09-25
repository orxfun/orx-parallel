use crate::Params;
use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::runner::ParallelRunner;
use crate::using::collect_into::u_par_collect_into::UParCollectIntoCore;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<O> UParCollectIntoCore<O> for FixedVec<O>
where
    O: Send + Sync,
{
    fn u_m_collect_into<U, R, I, M1>(
        self,
        using: U,
        orchestrator: R,
        params: Params,
        iter: I,
        map1: M1,
    ) -> Self
    where
        U: crate::using::using_variants::Using,
        R: ParallelRunner,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.u_m_collect_into(using, orchestrator, params, iter, map1))
    }

    fn u_x_collect_into<U, R, I, Vo, X1>(
        self,
        using: U,
        orchestrator: R,
        params: Params,
        iter: I,
        xap1: X1,
    ) -> Self
    where
        U: crate::using::using_variants::Using,
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: TransformableValues<Item = O, Fallibility = Infallible>,
        X1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.u_x_collect_into(using, orchestrator, params, iter, xap1))
    }
}
