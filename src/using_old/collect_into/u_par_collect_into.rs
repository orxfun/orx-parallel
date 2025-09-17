use crate::collect_into::ParCollectIntoCore;
use crate::generic_values::Values;
use crate::generic_values::runner_results::Infallible;
use crate::runner::ParallelRunner;
use crate::using_old::Using;
use crate::using_old::computations::{UM, UX};
use orx_concurrent_iter::ConcurrentIter;

pub trait UParCollectIntoCoreOld<O>: ParCollectIntoCore<O> {
    fn u_m_collect_into_old<R, U, I, M1>(self, m: UM<U, I, O, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Sync;

    fn u_x_collect_into_old<R, U, I, Vo, M1>(self, x: UX<U, I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vo: Values<Item = O, Fallibility = Infallible>,
        M1: Fn(&mut U::Item, I::Item) -> Vo + Sync;
}
