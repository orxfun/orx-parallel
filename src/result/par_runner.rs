use crate::infallible::Xap;
use crate::result::size_pairs::SizePairRes;
use crate::result::thread_execution as th;
use crate::results::{Val, ValIdx};
use crate::{parameters::Params, pool::ParThreadPool, runner::ParRunner};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParRunnerRes: ParRunner {
    fn next<I, M, E, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Result<Option<ValIdx<X2::O>>, E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
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
                        th::next::<Self, _, _, _, _, _, _>(sizes, th_idx, state, iter, x1, x2);
                    results.push(value);
                });
            }
        });

        ValIdx::first_res(results_bag.into_inner().into_inner())
    }

    fn next_any<I, M, E, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Result<Option<X2::O>, E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
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
                        th::next_any::<Self, _, _, _, _, _, _>(sizes, th_idx, state, iter, x1, x2);
                    results.push(value);
                });
            }
        });

        Val::first_res(results_bag.into_inner().into_inner())
    }

    fn reduce<I, M, E, X1, X2, S, F>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
        f: F,
    ) -> Result<Option<X2::O>, E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
        E: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, state, results) = (&iter, &state, &results_bag);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let value = th::reduce::<Self, _, _, _, _, _, _, _>(
                        sizes, th_idx, state, iter, x1, x2, f,
                    );
                    results.push(value);
                });
            }
        });

        Val::reduce_res(results_bag.into_inner().into_inner(), f)
    }

    fn collect<I, M, E, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Vec<Result<Vec<ValIdx<X2::O>>, E>>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M>,
        S: SizePairRes<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, state, results) = (&iter, &state, &results_bag);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let vec =
                        th::collect::<Self, _, _, _, _, _, _>(sizes, th_idx, state, iter, x1, x2);
                    results.push(vec);
                });
            }
        });

        results_bag.into_inner().into_inner()
    }
}

impl<R: ParRunner> ParRunnerRes for R {}
