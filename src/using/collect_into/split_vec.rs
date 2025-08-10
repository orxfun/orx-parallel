use crate::collect_into::utils::split_vec_reserve;
use crate::using::Using;
use crate::using::collect_into::u_par_collect_into::UParCollectIntoCore;
use crate::using::computations::{UM, UX};
use crate::{computations::Values, runner::ParallelRunner};
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{GrowthWithConstantTimeAccess, PseudoDefault, SplitVec};

impl<O, G> UParCollectIntoCore<O> for SplitVec<O, G>
where
    O: Send + Sync,
    G: GrowthWithConstantTimeAccess,
    Self: PseudoDefault,
{
    fn u_m_collect_into<R, U, I, M1>(mut self, m: UM<U, I, O, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    {
        split_vec_reserve(&mut self, m.par_len());
        let (_num_spawned, pinned_vec) = m.collect_into::<R, _>(self);
        pinned_vec
    }

    fn u_x_collect_into<R, U, I, Vo, M1>(mut self, x: UX<U, I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    {
        split_vec_reserve(&mut self, x.par_len());
        let (_num_spawned, pinned_vec) = x.collect_into::<R, _>(self);
        pinned_vec
    }
}
