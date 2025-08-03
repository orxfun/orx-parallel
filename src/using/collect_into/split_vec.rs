use crate::using::Using;
use crate::using::collect_into::u_par_collect_into::UParCollectIntoCore;
use crate::using::computations::{UM, UX, UXfx};
use crate::{computations::Values, runner::ParallelRunner};
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::PinnedVec;
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
        M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync,
    {
        reserve(&mut self, m.par_len());
        let (_num_spawned, pinned_vec) = m.collect_into::<R, _>(self);
        pinned_vec
    }

    fn u_x_collect_into<R, U, I, Vo, M1>(mut self, x: UX<U, I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vo: Values<Item = O> + Send + Sync,
        Vo::Item: Send + Sync,
        M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
    {
        reserve(&mut self, x.par_len());
        let (_num_spawned, pinned_vec) = x.collect_into::<R, _>(self);
        pinned_vec
    }

    fn u_xfx_collect_into<R, U, I, Vt, Vo, M1, F, M2>(
        mut self,
        xfx: UXfx<U, I, Vt, Vo, M1, F, M2>,
    ) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        Vt: Values + Send + Sync,
        Vt::Item: Send + Sync,
        Vo: Values<Item = O> + Send + Sync,
        M1: Fn(&mut U::Item, I::Item) -> Vt + Send + Sync,
        F: Fn(&mut U::Item, &Vt::Item) -> bool + Send + Sync,
        M2: Fn(&mut U::Item, Vt::Item) -> Vo + Send + Sync,
    {
        reserve(&mut self, xfx.par_len());
        let (_num_spawned, pinned_vec) = xfx.collect_into::<R, _>(self);
        pinned_vec
    }
}

fn reserve<T, G: GrowthWithConstantTimeAccess>(
    split_vec: &mut SplitVec<T, G>,
    len_to_extend: Option<usize>,
) {
    match len_to_extend {
        None => {
            let capacity_bound = split_vec.capacity_bound();
            split_vec.reserve_maximum_concurrent_capacity(capacity_bound)
        }
        Some(len) => split_vec.reserve_maximum_concurrent_capacity(split_vec.len() + len),
    };
}
