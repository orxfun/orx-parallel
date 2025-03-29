use super::{collect::XfxCollect, xfx::Xfx};
use crate::computations::Values;
use crate::runner::ParallelRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

impl<I, Vt, Vo, M1, F, M2> Xfx<I, Vt, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<Vo::Item>,
    {
        XfxCollect::compute::<R>(self, pinned_vec)
    }
}
