use crate::ParExtend;
use crate::infallible::thread_execution as th;
use crate::infallible::xap::Xap;
use crate::pool::{Scope, ThreadPool};
use crate::results::{Val, ValIdx};
use crate::{parameters::Params, runner::ParRunner};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;

pub(crate) trait ParRunnerInfallible: ParRunner {
    fn next<I, X>(&mut self, params: Params, iter: I, x: X) -> Option<ValIdx<X::O>>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
    {
        match params.is_sequential() {
            true => iter
                .into_seq_iter()
                .flat_map(|i| x.xap(i).into_iter())
                .enumerate()
                .next()
                .map(|(idx, val)| ValIdx::new(val, idx)),
            false => {
                let mut spawned = 0;
                let (max_nt, state) =
                    self.nt_state(params, I::is_source_serialized(), iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results, x) = (&iter, &state, &results_bag, x);
                self.pool_mut().scope(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        s.run(move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::next::<Self, _, _>(th_idx, st, iter, x);
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                ValIdx::first(results_bag.into_inner().into_inner())
            }
        }
    }

    fn next_any<I, X>(&mut self, params: Params, iter: I, x: X) -> Option<X::O>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
    {
        match params.is_sequential() {
            true => iter
                .into_seq_iter()
                .flat_map(|i| x.xap(i).into_iter())
                .next(),
            false => {
                let mut spawned = 0;
                let (max_nt, state) =
                    self.nt_state(params, I::is_source_serialized(), iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results, x) = (&iter, &state, &results_bag, x);
                self.pool_mut().scope(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        s.run(move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::next_any::<Self, _, _>(th_idx, st, iter, x);
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                Val::first(results_bag.into_inner().into_inner())
            }
        }
    }

    fn reduce<I, X, F>(&mut self, params: Params, iter: I, x: X, f: F) -> Option<X::O>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        F: Fn(X::O, X::O) -> X::O + Send + Copy,
        X::O: Send,
    {
        match params.is_sequential() {
            true => iter
                .into_seq_iter()
                .flat_map(|i| x.xap(i).into_iter())
                .reduce(f),
            _ => {
                let mut spawned = 0;
                let (max_nt, state) =
                    self.nt_state(params, I::is_source_serialized(), iter.size_hint(), None);
                let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

                let (iter, st, results, x) = (&iter, &state, &results_bag, x);
                self.pool_mut().scope(move |s| {
                    while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                        spawned += 1;
                        s.run(move || {
                            Self::begin_thread(st, th_idx);
                            let value = th::reduce::<Self, _, _, _>(th_idx, st, iter, x, f);
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                Val::reduce(results_bag.into_inner().into_inner(), f)
            }
        }
    }

    fn collect<I, X, P>(&mut self, params: Params, iter: I, x: X, dst: &mut P)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
        P: ParExtend<X::O>,
        P::OrderedThreadValues: Send,
    {
        match params.is_sequential() {
            true => {
                let values = iter.into_seq_iter().flat_map(|i| x.xap(i));
                dst.extend(values);
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
                            let value = th::collect::<Self, _, _, P>(th_idx, st, iter, x);
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                P::extend_merge_ordered_infallibles(dst, results_bag.into_inner().into_inner());
            }
        }
    }

    fn collect_arb<I, X, P>(&mut self, params: Params, iter: I, x: X, dst: &mut P)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
        P: ParExtend<X::O>,
        P::ThreadValues: Send,
    {
        match params.is_sequential() {
            true => {
                let values = iter.into_seq_iter().flat_map(|i| x.xap(i));
                dst.extend(values);
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
                            let value = th::collect_arb::<Self, _, _, P>(th_idx, st, iter, x);
                            results.push(value);
                            Self::complete_thread(st, th_idx);
                        });
                    }
                });

                Self::complete_computation(state);
                P::extend_merge_infallibles(dst, results_bag.into_inner().into_inner());
            }
        };
    }
}

impl<R: ParRunner> ParRunnerInfallible for R {}
