use crate::computations::Values;
use crate::runner::ParallelRunner;
use crate::using::Using;
use crate::using::collect_into::u_par_collect_into::UParCollectIntoCore;
use crate::using::computations::{UM, UX, UXfx};
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
        M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync,
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
        M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.u_x_collect_into::<R, _, _, _, _>(x))
    }

    fn u_xfx_collect_into<R, U, I, Vt, Vo, M1, F, M2>(
        self,
        xfx: UXfx<U, I, Vt, Vo, M1, F, M2>,
    ) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vt: Values,
        Vt::Item: Send + Sync,
        Vo: Values<Item = O>,
        M1: Fn(&mut U::Item, I::Item) -> Vt + Send + Sync,
        F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
        M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.u_xfx_collect_into::<R, _, _, _, _, _, _, _>(xfx))
    }
}
