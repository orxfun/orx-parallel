use super::{collect::XCollect, reduce::XReduce, x::X};
use crate::{computations::Values, runner::ParallelRunner};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::IntoConcurrentPinnedVec;

impl<I, Vo, M1> X<I, Vo, M1>
where
    I: ConcurrentIter,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vo + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        XCollect::compute::<R>(self, pinned_vec)
    }

    pub fn reduce<R, Red>(self, reduce: Red) -> (usize, Option<Vo::Item>)
    where
        R: ParallelRunner,
        Red: Fn(Vo::Item, Vo::Item) -> Vo::Item + Send + Sync,
    {
        XReduce::compute::<R>(self, reduce)
    }
}
