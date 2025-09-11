use super::par_collect_into::ParCollectIntoCore;
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
#[cfg(test)]
use orx_pinned_vec::PinnedVec;

impl<O> ParCollectIntoCore<O> for FixedVec<O>
where
    O: Send + Sync,
{
    type BridgePinnedVec = Self;

    fn empty(iter_len: Option<usize>) -> Self {
        let vec = <Vec<_> as ParCollectIntoCore<_>>::empty(iter_len);
        vec.into()
    }

    fn m_collect_into<R, I, M1>(self, m: ParMap<I, O, M1, R>) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Sync,
        O: Send,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.m_collect_into::<R, _, _>(m))
    }

    fn x_collect_into<R, I, Vo, X1>(self, x: ParXap<I, Vo, X1, R>) -> Self
    where
        R: Orchestrator,
        I: ConcurrentIter,
        Vo: TransformableValues<Item = O, Fallibility = Infallible>,
        X1: Fn(I::Item) -> Vo + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.x_collect_into::<R, _, _, _>(x))
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
        let vec = Vec::from(self);
        let result = vec.x_try_collect_into(x);
        result.map(FixedVec::from)
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
        let vec = Vec::from(self);
        let result = vec.x_try_collect_into_3(c);
        result.map(FixedVec::from)
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
    //     let vec = Vec::from(self);
    //     let result = vec.x_try_collect_into_2(x);
    //     result.map(FixedVec::from)
    // }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
