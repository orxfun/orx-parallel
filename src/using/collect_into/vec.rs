use crate::Params;
use crate::collect_into::utils::extend_vec_from_split;
use crate::generic_values::TransformableValues;
use crate::generic_values::runner_results::Infallible;
use crate::runner::ParallelRunner;
use crate::using::collect_into::collect::map_collect_into;
use crate::using::collect_into::u_par_collect_into::UParCollectIntoCore;
use crate::using::using_variants::Using;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;

impl<O> UParCollectIntoCore<O> for Vec<O>
where
    O: Send + Sync,
{
    fn u_m_collect_into<'using, U, R, I, M1>(
        mut self,
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
        M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    {
        match iter.try_get_len() {
            None => {
                let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
                let split_vec = split_vec.u_m_collect_into(using, orchestrator, params, iter, map1);
                extend_vec_from_split(self, split_vec)
            }
            Some(len) => {
                self.reserve(len);
                let fixed_vec = FixedVec::from(self);
                let (_, fixed_vec) =
                    map_collect_into(using, orchestrator, params, iter, map1, fixed_vec);
                Vec::from(fixed_vec)
            }
        }
    }

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
        X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let split_vec = split_vec.u_x_collect_into(using, orchestrator, params, iter, xap1);
        extend_vec_from_split(self, split_vec)
    }

    fn u_x_try_collect_into<'using, U, R, I, Vo, X1>(
        self,
        using: U,
        orchestrator: R,
        params: Params,
        iter: I,
        xap1: X1,
    ) -> Result<Self, <Vo::Fallibility as crate::generic_values::runner_results::Fallibility>::Error>
    where
        U: Using<'using>,
        R: ParallelRunner,
        I: ConcurrentIter,
        X1: Fn(*mut U::Item, I::Item) -> Vo + Sync,
        Vo: crate::generic_values::Values<Item = O>,
        Self: Sized,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let result = split_vec.u_x_try_collect_into(using, orchestrator, params, iter, xap1);
        result.map(|split_vec| extend_vec_from_split(self, split_vec))
    }
}
