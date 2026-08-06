use crate::infallible::Xap;
use crate::option::thread_execution as th;
use crate::results::{Val, ValIdx, ValsAndIdx};
use crate::sizes::SizePair;
use crate::{parameters::Params, pool::ParThreadPool, runner::ParRunner};
use alloc::vec;
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParRunnerOpt: ParRunner {
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
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
    {
        match params.is_sequential() {
            true => {
                let first = iter
                    .into_seq_iter()
                    .flat_map(|i| S::xap_opt(x1, x2, i).into_iter())
                    .enumerate()
                    .next();
                match first {
                    None => Some(None),
                    Some((idx, result)) => match result {
                        Some(val) => Some(Some(ValIdx { val, idx })),
                        None => None,
                    },
                }
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scoped_computation(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                            Self::begin_thread(st, th_idx);
                            let value =
                                th::next::<Self, _, _, _, _, _>(sizes, th_idx, st, iter, x1, x2);
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                ValIdx::first_opt(results_bag.into_inner().into_inner())
            }
        }
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
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
    {
        match params.is_sequential() {
            true => {
                let first = iter
                    .into_seq_iter()
                    .flat_map(|i| S::xap_opt(x1, x2, i).into_iter())
                    .next();
                match first {
                    None => Some(None),
                    Some(result) => result.map(Some),
                }
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scoped_computation(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::next_any::<Self, _, _, _, _, _>(
                                sizes, th_idx, st, iter, x1, x2,
                            );
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Val::first_opt(results_bag.into_inner().into_inner())
            }
        }
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
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
    {
        match params.is_sequential() {
            true => {
                let mut iter = iter
                    .into_seq_iter()
                    .flat_map(|i| S::xap_opt(x1, x2, i).into_iter());
                match iter.next() {
                    None => Some(None),
                    Some(None) => None,
                    Some(Some(mut acc)) => {
                        for maybe in iter {
                            acc = f(acc, maybe?);
                        }
                        Some(Some(acc))
                    }
                }
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scoped_computation(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::reduce::<Self, _, _, _, _, _, _>(
                                sizes, th_idx, st, iter, x1, x2, f,
                            );
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Val::reduce_opt(results_bag.into_inner().into_inner(), f)
            }
        }
    }

    fn collect<I, M, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Option<Vec<ValsAndIdx<X2::O>>>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
    {
        match params.is_sequential() {
            true => {
                let iter = iter
                    .into_seq_iter()
                    .flat_map(|i| S::xap_opt(x1, x2, i).into_iter());
                let mut values = vec![];
                for maybe in iter {
                    values.push(maybe?);
                }
                Some(vec![ValsAndIdx::new_seq(values)])
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scoped_computation(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                            Self::begin_thread(st, th_idx);
                            let value =
                                th::collect::<Self, _, _, _, _, _>(sizes, th_idx, st, iter, x1, x2);
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                results_bag.into_inner().into_inner().into_iter().collect()
            }
        }
    }

    fn collect_arb<I, M, X1, X2, S>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
    ) -> Option<Vec<Vec<X2::O>>>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Option<M>>,
        X2: Xap<I = M>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
    {
        match params.is_sequential() {
            true => {
                let iter = iter
                    .into_seq_iter()
                    .flat_map(|i| S::xap_opt(x1, x2, i).into_iter());
                let mut values = vec![];
                for maybe in iter {
                    values.push(maybe?);
                }
                Some(vec![values])
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scoped_computation(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::collect_arb::<Self, _, _, _, _, _>(
                                sizes, th_idx, st, iter, x1, x2,
                            );
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                results_bag.into_inner().into_inner().into_iter().collect()
            }
        }
    }
}

impl<R: ParRunner> ParRunnerOpt for R {}
