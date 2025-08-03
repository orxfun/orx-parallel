use super::par_collect_into::ParCollectIntoCore;
use crate::computations::{M, UM, UX, UXfx, Values, X, Xfx};
use crate::runner::ParallelRunner;
use crate::using::Using;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

impl<O> ParCollectIntoCore<O> for Vec<O>
where
    O: Send + Sync,
{
    type BridgePinnedVec = FixedVec<O>;

    fn empty(iter_len: Option<usize>) -> Self {
        match iter_len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        }
    }

    fn m_collect_into<R, I, M1>(mut self, m: M<I, O, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Send + Sync,
    {
        match m.par_len() {
            None => {
                let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
                let split_vec = split_vec.m_collect_into::<R, _, _>(m);
                extend_from_split(self, split_vec)
            }
            Some(len) => {
                self.reserve(len);
                let fixed_vec = FixedVec::from(self);
                let (_num_spawned, fixed_vec) = m.collect_into::<R, _>(fixed_vec);
                Vec::from(fixed_vec)
            }
        }
    }

    fn x_collect_into<R, I, Vo, M1>(self, x: X<I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O> + Send + Sync,
        Vo::Item: Send + Sync,
        M1: Fn(I::Item) -> Vo + Send + Sync,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let split_vec = split_vec.x_collect_into::<R, _, _, _>(x);
        extend_from_split(self, split_vec)
    }

    fn xfx_collect_into<R, I, Vt, Vo, M1, F, M2>(self, xfx: Xfx<I, Vt, Vo, M1, F, M2>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vt: Values + Send + Sync,
        Vt::Item: Send + Sync,
        Vo: Values<Item = O> + Send + Sync,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&Vt::Item) -> bool + Send + Sync,
        M2: Fn(Vt::Item) -> Vo + Send + Sync,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let split_vec = split_vec.xfx_collect_into::<R, _, _, _, _, _, _>(xfx);
        extend_from_split(self, split_vec)
    }

    fn u_m_collect_into<R, U, I, M1>(mut self, m: UM<U, I, O, M1>) -> Self
    where
        R: ParallelRunner,
        U: Using,
        I: ConcurrentIter,
        M1: Fn(&mut U::Item, I::Item) -> O + Send + Sync,
    {
        match m.par_len() {
            None => {
                let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
                let split_vec = split_vec.u_m_collect_into::<R, _, _, _>(m);
                extend_from_split(self, split_vec)
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
        Vo: Values<Item = O> + Send + Sync,
        Vo::Item: Send + Sync,
        M1: Fn(&mut U::Item, I::Item) -> Vo + Send + Sync,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let split_vec = split_vec.u_x_collect_into::<R, _, _, _, _>(x);
        extend_from_split(self, split_vec)
    }

    fn u_xfx_collect_into<R, U, I, Vt, Vo, M1, F, M2>(
        self,
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
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let split_vec = split_vec.u_xfx_collect_into::<R, _, _, _, _, _, _, _>(xfx);
        extend_from_split(self, split_vec)
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}

fn extend_from_split<T, G>(mut initial_vec: Vec<T>, collected_split_vec: SplitVec<T, G>) -> Vec<T>
where
    G: GrowthWithConstantTimeAccess,
{
    match initial_vec.len() {
        0 => collected_split_vec.to_vec(),
        _ => {
            initial_vec.reserve(collected_split_vec.len());
            initial_vec.extend(collected_split_vec);
            initial_vec
        }
    }
}
