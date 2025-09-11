use crate::collect_into::utils::extend_vec_from_split;
use crate::generic_values::Values;
use crate::generic_values::runner_results::Infallible;
use crate::runner::ParallelRunner;
use crate::using::Using;
use crate::using::collect_into::u_par_collect_into::UParCollectIntoCore;
use crate::using::computations::{UM, UX};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;

impl<O> UParCollectIntoCore<O> for Vec<O>
where
    O: Send + Sync,
{
    fn u_m_collect_into<R, U, I, M1>(mut self, m: UM<U, I, O, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    {
        match m.iter().try_get_len() {
            None => {
                let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
                let split_vec = split_vec.u_m_collect_into::<R, _, _, _>(m);
                extend_vec_from_split(self, split_vec)
            }
            Some(len) => {
                self.reserve(len);
                let fixed_vec = FixedVec::from(self);
                let (_num_spawned, fixed_vec) = m.collect_into::<R, _>(fixed_vec);
                Vec::from(fixed_vec)
            }
        }
    }

    fn u_x_collect_into<R, U, I, Vo, M1>(self, x: UX<U, I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vo: Values<Item = O, Fallibility = Infallible>,
        M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let split_vec = split_vec.u_x_collect_into::<R, _, _, _, _>(x);
        extend_vec_from_split(self, split_vec)
    }
}
