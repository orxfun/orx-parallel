use super::par_collect_into::ParCollectIntoCore;
use crate::collect_into::utils::extend_vec_from_split;
use crate::computational_variants::fallible_result::ParXapResult;
use crate::computational_variants::fallible_result::computations::{ParResultCollectInto, X};
use crate::computational_variants::{ParMap, ParXap};
use crate::generic_values::runner_results::{Fallibility, Fallible, Infallible};
use crate::generic_values::{TransformableValues, Values};
use crate::orch::Orchestrator;
use crate::par_iter_result::IntoResult;
use crate::runner::ParallelRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;

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

    fn m_collect_into<R, I, M1>(mut self, m: ParMap<I, O, M1, R>) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Sync,
        O: Send,
    {
        match m.par_len() {
            None => {
                let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
                let split_vec = split_vec.m_collect_into(m);
                extend_vec_from_split(self, split_vec)
            }
            Some(len) => {
                self.reserve(len);
                let fixed_vec = FixedVec::from(self);
                let (_, fixed_vec) = m.par_collect_into(fixed_vec);
                Vec::from(fixed_vec)
            }
        }
    }

    fn x_collect_into<R, I, Vo, X1>(self, x: ParXap<I, Vo, X1, R>) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        Vo: TransformableValues<Item = O, Fallibility = Infallible>,
        X1: Fn(I::Item) -> Vo + Sync,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let split_vec = split_vec.x_collect_into::<R, _, _, _>(x);
        extend_vec_from_split(self, split_vec)
    }

    fn x_try_collect_into<R, I, Vo, M1>(
        self,
        x: X<R, I, Vo, M1>,
    ) -> Result<Self, <Vo::Fallibility as Fallibility>::Error>
    where
        R: Orchestrator,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vo + Sync,
        Self: Sized,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let result = split_vec.x_try_collect_into(x);
        result.map(|split_vec| extend_vec_from_split(self, split_vec))
    }

    fn x_try_collect_into_3<I, E, Vo, X1, R>(
        self,
        c: ParResultCollectInto<R, I, O, E, Vo, X1>,
    ) -> Result<Self, E>
    where
        R: Orchestrator,
        I: ConcurrentIter,
        Vo: TransformableValues,
        Vo::Item: IntoResult<O, E>,
        X1: Fn(I::Item) -> Vo + Sync,
        O: Send,
        E: Send,
        Self: Sized,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let result = split_vec.x_try_collect_into_3(c);
        result.map(|split_vec| extend_vec_from_split(self, split_vec))
    }

    // fn x_try_collect_into_2<I, E, Vo, X1, R>(
    //     self,
    //     x: ParXapResult<I, O, E, Vo, X1, R>,
    // ) -> Result<Self, <Vo::Fallibility as Fallibility>::Error>
    // where
    //     R: Orchestrator,
    //     I: ConcurrentIter,
    //     Vo: TransformableValues<Fallibility = Fallible<E>>,
    //     X1: Fn(I::Item) -> Vo + Sync,
    //     Vo::Item: IntoResult<O, E> + Send,
    //     E: Send,
    //     Self: Sized,
    // {
    //     let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
    //     let result = split_vec.x_try_collect_into_2(x);
    //     result.map(|split_vec| extend_vec_from_split(self, split_vec))
    // }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
