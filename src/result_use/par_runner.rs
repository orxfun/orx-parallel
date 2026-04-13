use crate::infallible_use::{Use, XapUse};
use crate::result_use::size_pairs::SizePairUseRes;
use crate::result_use::thread_execution as th;
use crate::results::{Val, ValIdx, ValsAndIdx};
use crate::{parameters::Params, pool::ParThreadPool, runner::ParRunner};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParRunnerUseRes: ParRunner {
    fn next<U, I, M, E, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        u: U,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Result<Option<ValIdx<X2::O>>, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M>,
        S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, u) = (&iter, &state, &results_bag, &u);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let value =
                        th::next::<Self, _, _, _, _, _, _, _>(sizes, u, th_idx, st, iter, x1, x2);
                    results.push(value);
                });
            }
        });

        Self::complete_computation(state);
        ValIdx::first_res(results_bag.into_inner().into_inner())
    }

    fn next_any<U, I, M, E, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        u: U,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Result<Option<X2::O>, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M>,
        S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, u) = (&iter, &state, &results_bag, &u);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let value = th::next_any::<Self, _, _, _, _, _, _, _>(
                        sizes, u, th_idx, st, iter, x1, x2,
                    );
                    results.push(value);
                });
            }
        });

        Self::complete_computation(state);
        Val::first_res(results_bag.into_inner().into_inner())
    }

    fn reduce<U, I, M, E, X1, X2, S, F>(
        &mut self,
        sizes: S,
        params: Params,
        u: U,
        iter: I,
        x1: X1,
        x2: X2,
        f: F,
    ) -> Result<Option<X2::O>, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M>,
        S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
        F: Fn(&mut U::Item, X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
        E: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        {
            let (iter, st, results, u) = (&iter, &state, &results_bag, &u);
            self.pool_mut().scoped_computation(move |s| {
                while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                    spawned += 1;
                    <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                        let value = th::reduce::<Self, _, _, _, _, _, _, _, _>(
                            sizes, u, th_idx, st, iter, x1, x2, f,
                        );
                        results.push(value);
                    });
                }
            });
        }

        Self::complete_computation(state);
        let mut u = u.into_inner();
        Val::reduce_res(results_bag.into_inner().into_inner(), |a, b| {
            f(&mut u, a, b)
        })
    }

    fn collect<U, I, M, E, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        u: U,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Result<Vec<ValsAndIdx<X2::O>>, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M>,
        S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, u) = (&iter, &state, &results_bag, &u);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let vec = th::collect::<Self, _, _, _, _, _, _, _>(
                        sizes, u, th_idx, st, iter, x1, x2,
                    );
                    results.push(vec);
                });
            }
        });

        Self::complete_computation(state);
        results_bag.into_inner().into_inner().into_iter().collect()
    }

    fn collect_arb<U, I, M, E, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        u: U,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Result<Vec<Vec<X2::O>>, E>
    where
        U: Use,
        I: ConcurrentIter,
        X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
        X2: XapUse<U = U::Item, I = M>,
        S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, u) = (&iter, &state, &results_bag, &u);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let vec = th::collect_arb::<Self, _, _, _, _, _, _, _>(
                        sizes, u, th_idx, st, iter, x1, x2,
                    );
                    results.push(vec);
                });
            }
        });

        Self::complete_computation(state);
        results_bag.into_inner().into_inner().into_iter().collect()
    }
}

impl<R: ParRunner> ParRunnerUseRes for R {}
