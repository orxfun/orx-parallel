use crate::runner::ParallelRunner;
use crate::using::Using;
use crate::using::collect_into::u_par_collect_into::UParCollectIntoCore;
use crate::using::computations::{UM, UX};
use crate::values::Values;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<O> UParCollectIntoCore<O> for FixedVec<O>
where
    O: Send + Sync,
{
    fn u_m_collect_into<R, U, I, M1>(self, m: UM<U, I, O, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.u_m_collect_into::<R, _, _, _>(m))
    }

    fn u_x_collect_into<R, U, I, Vo, M1>(self, x: UX<U, I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        Vo::Item: Send + Sync,
        M1: Fn(&mut U::Item, I::Item) -> Vo + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.u_x_collect_into::<R, _, _, _, _>(x))
    }
}
