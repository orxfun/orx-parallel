use crate::infallible_use::XapUse;
use crate::infallible_use::thread_execution as th;
use crate::infallible_use::use_var::Use;
use crate::infallible_use::use_var::UseVec;
use crate::results::ValsAndIdx;
use crate::results::{Val, ValIdx};
use crate::{parameters::Params, pool::ParThreadPool, runner::ParRunner};
use alloc::vec::Vec;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_self_or::SoM;

pub trait ParRunnerInfallibleUse: ParRunner {
    fn next<U, I, X>(&mut self, params: Params, u: U, iter: I, x: X) -> Option<ValIdx<X::O>>
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item>,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, x, u) = (&iter, &state, &results_bag, x, &u);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    Self::begin_thread(st, th_idx);
                    let mut u = u.get(th_idx);
                    let value = th::next::<Self, _, _, _>(u.get_mut(), th_idx, st, iter, x);
                    results.push(value);
                    Self::complete_thread(st, th_idx);
                });
            }
        });

        Self::complete_computation(state);
        ValIdx::first(results_bag.into_inner().into_inner())
    }

    fn next_any<U, I, X>(&mut self, params: Params, u: U, iter: I, x: X) -> Option<X::O>
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item>,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, x, u) = (&iter, &state, &results_bag, x, &u);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    Self::begin_thread(st, th_idx);
                    let mut u = u.get(th_idx);
                    let value = th::next_any::<Self, _, _, _>(u.get_mut(), th_idx, st, iter, x);
                    results.push(value);
                    Self::complete_thread(st, th_idx);
                });
            }
        });

        Self::complete_computation(state);
        Val::first(results_bag.into_inner().into_inner())
    }

    fn reduce<U, I, X, F>(&mut self, params: Params, u: U, iter: I, x: X, f: F) -> Option<X::O>
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item>,
        F: Fn(&mut X::U, X::O, X::O) -> X::O + Send + Copy,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        {
            let (iter, st, results, x, u) = (&iter, &state, &results_bag, x, &u);
            self.pool_mut().scoped_computation(move |s| {
                while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                    spawned += 1;
                    <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                        Self::begin_thread(st, th_idx);
                        let mut u = u.get(th_idx);
                        let value =
                            th::reduce::<Self, _, _, _, _>(u.get_mut(), th_idx, st, iter, x, f);
                        results.push(value);
                        Self::complete_thread(st, th_idx);
                    });
                }
            });
        }

        Self::complete_computation(state);
        let mut u = u.create(results_bag.len());
        Val::reduce(results_bag.into_inner().into_inner(), |a, b| {
            f(&mut u, a, b)
        })
    }

    fn fold<U, I, X, F>(&mut self, params: Params, u: U, iter: I, x: X, f: F) -> Vec<U::Item>
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item, O = ()>,
        F: Fn(&mut X::U, X::O, X::O) + Send + Copy,
        U::Item: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        {
            let (iter, st, results, x, u) = (&iter, &state, &results_bag, x, &u);
            self.pool_mut().scoped_computation(move |s| {
                while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                    spawned += 1;
                    <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                        Self::begin_thread(st, th_idx);
                        let (_, u) =
                            th::reduce_get_u::<Self, _, _, _, _>(u, th_idx, st, iter, x, f);
                        results.push(u);
                        Self::complete_thread(st, th_idx);
                    });
                }
            });
        }

        Self::complete_computation(state);
        results_bag.into_inner().into_inner()
    }

    fn collect<U, I, X>(&mut self, params: Params, u: U, iter: I, x: X) -> Vec<ValsAndIdx<X::O>>
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item>,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, u) = (&iter, &state, &results_bag, &u);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    Self::begin_thread(st, th_idx);
                    let mut u = u.get(th_idx);
                    let value = th::collect::<Self, _, _, _>(u.get_mut(), th_idx, st, iter, x);
                    results.push(value);
                    Self::complete_thread(st, th_idx);
                });
            }
        });

        Self::complete_computation(state);
        results_bag.into_inner().into_inner()
    }

    fn collect_arb<U, I, X>(&mut self, params: Params, u: U, iter: I, x: X) -> Vec<Vec<X::O>>
    where
        U: Use,
        I: ConcurrentIter,
        X: XapUse<U = U::Item, I = I::Item>,
        X::O: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.size_hint());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, st, results, u) = (&iter, &state, &results_bag, &u);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, st) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    Self::begin_thread(st, th_idx);
                    let mut u = u.get(th_idx);
                    let value = th::collect_arb::<Self, _, _, _>(u.get_mut(), th_idx, st, iter, x);
                    results.push(value);
                    Self::complete_thread(st, th_idx);
                });
            }
        });

        Self::complete_computation(state);
        results_bag.into_inner().into_inner()
    }
}

impl<R: ParRunner> ParRunnerInfallibleUse for R {}
