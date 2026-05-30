use crate::infallible::thread_execution as th;
use crate::infallible::xap::Xap;
use crate::results::{Val, ValIdx, ValsAndIdx};
use crate::{parameters::Params, pool::ParThreadPool, runner::ParRunner};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParRunnerInfallible: ParRunner {
    fn next<I, X>(&mut self, params: Params, iter: I, x: X) -> Option<ValIdx<X::O>>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, x) = (&iter, &state, &results_bag, x);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
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

    fn next_any<I, X>(&mut self, params: Params, iter: I, x: X) -> Option<X::O>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, x) = (&iter, &state, &results_bag, x);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
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

    fn reduce<I, X, F>(&mut self, params: Params, iter: I, x: X, f: F) -> Option<X::O>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        F: Fn(X::O, X::O) -> X::O + Send + Copy,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, x) = (&iter, &state, &results_bag, x);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
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

    fn collect<I, X>(&mut self, params: Params, iter: I, x: X) -> Vec<ValsAndIdx<X::O>>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results) = (&iter, &state, &results_bag);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    Self::begin_thread(st, th_idx);
                    let value = th::collect::<Self, _, _>(th_idx, st, iter, x);
                    results.push(value);
                    Self::complete_thread(st, th_idx);
                });
            }
        });

        Self::complete_computation(state);
        results_bag.into_inner().into_inner()
    }

    fn collect_arb<I, X>(&mut self, params: Params, iter: I, x: X) -> Vec<Vec<X::O>>
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint(), None);
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results) = (&iter, &state, &results_bag);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    Self::begin_thread(st, th_idx);
                    let value = th::collect_arb::<Self, _, _>(th_idx, st, iter, x);
                    results.push(value);
                    Self::complete_thread(st, th_idx);
                });
            }
        });

        Self::complete_computation(state);
        results_bag.into_inner().into_inner()
    }

    #[cfg(feature = "experimental")]
    fn collect_arb_over_bag<I, X, P>(&mut self, params: Params, iter: I, x: X, pinned_vec: P) -> P
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
        P: orx_pinned_vec::IntoConcurrentPinnedVec<X::O>,
    {
        let capacity_bound = pinned_vec.capacity_bound();
        let offset = pinned_vec.len();
        let mut results_bag: ConcurrentBag<X::O, P> = pinned_vec.into();
        match iter.try_get_len() {
            Some(iter_len) => results_bag.reserve_maximum_capacity(offset + iter_len),
            None => results_bag.reserve_maximum_capacity(capacity_bound),
        };

        let mut spawned = 0;
        let (_, state) = self.nt_state(params, iter.size_hint(), None);

        let (iter, st, bag) = (&iter, &state, &results_bag);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    Self::begin_thread(st, th_idx);
                    th::collect_arb_over_bag::<Self, _, _, _>(th_idx, st, iter, x, bag);
                    Self::complete_thread(st, th_idx);
                });
            }
        });

        Self::complete_computation(state);
        results_bag.into_inner()
    }
}

impl<R: ParRunner> ParRunnerInfallible for R {}
