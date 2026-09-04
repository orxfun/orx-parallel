use crate::ParExtend;
use crate::infallible::Xap;
use crate::pool::{Scope, ThreadPool};
use crate::result::thread_execution as th;
use crate::results::{Val, ValIdx};
use crate::sizes::SizePair;
use crate::{parameters::Params, runner::ParRunner};
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
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
    {
        match params.is_sequential() {
            true => {
                let first = iter
                    .into_seq_iter()
                    .flat_map(|i| S::xap_res(x1, x2, i).into_iter())
                    .enumerate()
                    .next();
                match first {
                    None => Ok(None),
                    Some((idx, result)) => match result {
                        Ok(val) => Ok(Some(ValIdx { val, idx })),
                        Err(e) => Err(e),
                    },
                }
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) =
                    self.nt_state(params, I::is_source_serialized(), iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scope(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        s.run(move || {
                            Self::begin_thread(st, th_idx);
                            let value =
                                th::next::<Self, _, _, _, _, _, _>(sizes, th_idx, st, iter, x1, x2);
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                ValIdx::first_res(results_bag.into_inner().into_inner())
            }
        }
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
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
    {
        match params.is_sequential() {
            true => {
                let first = iter
                    .into_seq_iter()
                    .flat_map(|i| S::xap_res(x1, x2, i).into_iter())
                    .next();
                match first {
                    None => Ok(None),
                    Some(result) => match result {
                        Ok(val) => Ok(Some(val)),
                        Err(e) => Err(e),
                    },
                }
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) =
                    self.nt_state(params, I::is_source_serialized(), iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scope(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        s.run(move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::next_any::<Self, _, _, _, _, _, _>(
                                sizes, th_idx, st, iter, x1, x2,
                            );
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                Val::first_res(results_bag.into_inner().into_inner())
            }
        }
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
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
        E: Send,
    {
        match params.is_sequential() {
            true => {
                let mut iter = iter
                    .into_seq_iter()
                    .flat_map(|i| S::xap_res(x1, x2, i).into_iter());
                match iter.next() {
                    None => Ok(None),
                    Some(Err(e)) => Err(e),
                    Some(Ok(mut acc)) => {
                        for maybe in iter {
                            acc = f(acc, maybe?);
                        }
                        Ok(Some(acc))
                    }
                }
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) =
                    self.nt_state(params, I::is_source_serialized(), iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scope(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        s.run(move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::reduce::<Self, _, _, _, _, _, _, _>(
                                sizes, th_idx, st, iter, x1, x2, f,
                            );
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                Val::reduce_res(results_bag.into_inner().into_inner(), f)
            }
        }
    }

    fn collect<I, M, E, X1, X2, S, P>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
        dst: &mut P,
    ) -> Result<(), E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
        P: ParExtend<X2::O>,
        P::OrderedThreadValues: Send,
    {
        match params.is_sequential() {
            true => {
                let fallibles = iter.into_seq_iter().flat_map(|i| S::xap_res(x1, x2, i));
                dst.extend_fallibles(fallibles)
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) =
                    self.nt_state(params, I::is_source_serialized(), iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scope(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        s.run(move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::collect::<Self, _, _, _, _, _, _, P>(
                                sizes, th_idx, st, iter, x1, x2,
                            );
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                P::extend_merge_ordered_fallibles(dst, results_bag.into_inner().into_inner())
            }
        }
    }

    fn collect_arb<I, M, E, X1, X2, S, P>(
        &mut self,
        sizes: S,
        params: Params,
        iter: I,
        x1: X1,
        x2: X2,
        dst: &mut P,
    ) -> Result<(), E>
    where
        I: ConcurrentIter,
        X1: Xap<I = I::Item, O = Result<M, E>>,
        X2: Xap<I = M>,
        S: SizePair<S1 = X1::Size, S2 = X2::Size>,
        X2::O: Send,
        E: Send,
        P: ParExtend<X2::O>,
        P::ThreadValues: Send,
    {
        match params.is_sequential() {
            true => {
                let fallibles = iter.into_seq_iter().flat_map(|i| S::xap_res(x1, x2, i));
                dst.extend_fallibles(fallibles)
            }
            false => {
                let mut spawned = 0;
                let (max_nt, state) =
                    self.nt_state(params, I::is_source_serialized(), iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results) = (&iter, &state, &results_bag);
                self.pool_mut().scope(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        s.run(move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::collect_arb::<Self, _, _, _, _, _, _, P>(
                                sizes, th_idx, st, iter, x1, x2,
                            );
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                P::extend_merge_fallibles(dst, results_bag.into_inner().into_inner())
            }
        }
    }
}

impl<R: ParRunner> ParRunnerRes for R {}
