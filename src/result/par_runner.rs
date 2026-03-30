use crate::result::thread_execution as th;
use crate::result::xap_res::XapRes;
use crate::results::ValIdx;
use crate::{parameters::Params, pool::ParThreadPool, runner::ParRunner};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParRunnerResult: ParRunner {
    fn next<I, X>(&mut self, params: Params, iter: I, x: X) -> Result<Option<ValIdx<X::O>>, X::E>
    where
        I: ConcurrentIter,
        X: XapRes<I = I::Item>,
        X::O: Send,
        X::E: Send,
    {
        let mut spawned = 0;
        let (max_nt, state) = self.nt_state(params, iter.try_get_len());
        let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

        let (iter, state, results, x) = (&iter, &state, &results_bag, x);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let value = th::next::<Self, _, _>(th_idx, state, iter, x);
                    results.push(value);
                });
            }
        });

        ValIdx::first_res(results_bag.into_inner().into_inner())
    }

    //     fn next_any<I, X>(&mut self, params: Params, iter: I, x: X) -> Option<X::O>
    //     where
    //         I: ConcurrentIter,
    //         X: XapRes<I = I::Item>,
    //         X::O: Send,
    //     {
    //         let mut spawned = 0;
    //         let (max_nt, state) = self.nt_state(params, iter.try_get_len());
    //         let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

    //         let (iter, state, results, x) = (&iter, &state, &results_bag, x);
    //         self.pool_mut().scoped_computation(move |s| {
    //             while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
    //                 spawned += 1;
    //                 <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
    //                     let value = th::next_any::<Self, _, _>(th_idx, state, iter, x);
    //                     results.push(value);
    //                 });
    //             }
    //         });

    //         results_bag.into_inner().into_iter().flatten().next()
    //     }

    //     fn reduce<I, X, F>(&mut self, params: Params, iter: I, x: X, f: F) -> Option<X::O>
    //     where
    //         I: ConcurrentIter,
    //         X: XapRes<I = I::Item>,
    //         F: Fn(X::O, X::O) -> X::O + Send + Copy,
    //         X::O: Send,
    //     {
    //         let mut spawned = 0;
    //         let (max_nt, state) = self.nt_state(params, iter.try_get_len());
    //         let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

    //         let (iter, state, results, x) = (&iter, &state, &results_bag, x);
    //         self.pool_mut().scoped_computation(move |s| {
    //             while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
    //                 spawned += 1;
    //                 <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
    //                     let value = th::reduce::<Self, _, _, _>(th_idx, state, iter, x, f);
    //                     results.push(value);
    //                 });
    //             }
    //         });

    //         results_bag.into_inner().into_iter().flatten().reduce(f)
    //     }
}

impl<R: ParRunner> ParRunnerResult for R {}
