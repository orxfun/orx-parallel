use crate::infallible::Xap;
use crate::option::size_pairs::SizePairOpt;
use crate::option::thread_execution as th;
use crate::results::{Val, ValIdx};
use crate::{parameters::Params, pool::ParThreadPool, runner::ParRunner};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParRunnerOption: ParRunner {
    fn next<I, M, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Option<Option<ValIdx<X2::O>>>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M>,
        S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, state, results) = (&iter, &state, &results_bag);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let value = th::next::<Self, _, _, _, _, _>(sizes, th_idx, state, iter, x1, x2);
                    results.push(value);
                });
            }
        });

        ValIdx::first_opt(results_bag.into_inner().into_inner())
    }

    fn next_any<I, M, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Option<Option<X2::O>>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M>,
        S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, state, results) = (&iter, &state, &results_bag);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let value =
                        th::next_any::<Self, _, _, _, _, _>(sizes, th_idx, state, iter, x1, x2);
                    results.push(value);
                });
            }
        });

        Val::first_opt(results_bag.into_inner().into_inner())
    }

    fn reduce<I, M, X1, X2, S, F>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
        f: F,
    ) -> Option<Option<X2::O>>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M>,
        S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
        F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, state, results) = (&iter, &state, &results_bag);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let value =
                        th::reduce::<Self, _, _, _, _, _, _>(sizes, th_idx, state, iter, x1, x2, f);
                    results.push(value);
                });
            }
        });

        Val::reduce_opt(results_bag.into_inner().into_inner(), f)
    }
}

impl<R: ParRunner> ParRunnerOption for R {}
