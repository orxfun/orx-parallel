use super::{collect::MfmCollect, mfm::Mfm};
use crate::computations::Values;
use crate::runner::ParallelRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

impl<I, T, Vt, O, Vo, M1, F, M2> Mfm<I, T, Vt, O, Vo, M1, F, M2>
where
    I: ConcurrentIter,
    T: Send + Sync,
    Vt: Values<Item = T> + Send + Sync,
    O: Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
{
    pub fn collect_into<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<O>,
    {
        MfmCollect::compute::<R>(self, pinned_vec)
    }
}
