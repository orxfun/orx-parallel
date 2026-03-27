use crate::parameters::Params;
use crate::runner::results::{Opt, OptIdx, ValIdx};
use crate::runner::thread_computations as th;
use crate::{pool::ParThreadPool, xap::Xap};
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;

pub trait ParRunner: Sized + Sync {
    /// Underlying thread pool.
    type Pool: ParThreadPool;

    /// Parallel computation state that is shared among thread computations.
    type State: Send + Sync;

    type ChunkState;

    // required

    /// Reference to the underlying thread pool.
    fn pool(&self) -> &Self::Pool;

    /// Mutable reference to the underlying thread pool.
    fn pool_mut(&mut self) -> &mut Self::Pool;

    // required - orchestration

    /// Returns Some of the index of the new thread if it is beneficial to spawn a new thread
    /// for the remaining computation; it returns None otherwise.
    ///
    /// Since this method is called sequentially, returned value will always be `Some(spawned)` if
    /// it returns some.
    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize>;

    /// Creates an initial state for a new parallel computation.
    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State;

    /// Returns the next chunk size to be pulled from the input with `remaining` length
    /// for the current `state`.
    fn next_chunk_size(state: &Self::State, remaining: Option<usize>) -> usize;

    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState;

    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState);

    fn complete_computation(state: Self::State);

    // // provided

    // fn next<I, X>(&mut self, params: Params, iter: I, x: X) -> Option<ValIdx<X::O>>
    // where
    //     I: ConcurrentIter,
    //     X: Xap<I = I::Item>,
    //     X::O: Send,
    // {
    //     let mut spawned = 0;
    //     let (max_nt, state) = self.nt_state(params, iter.try_get_len());
    //     let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

    //     let (iter, state, results, x) = (&iter, &state, &results_bag, x);
    //     self.pool_mut().scoped_computation(move |s| {
    //         while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
    //             spawned += 1;
    //             <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
    //                 let value = th::next::<Self, _, _>(th_idx, state, iter, x);
    //                 results.push(value);
    //             });
    //         }
    //     });

    //     ValIdx::first_of(results_bag.into_inner().into_inner())
    // }

    // fn next_any<I, X>(&mut self, params: Params, iter: I, x: X) -> Option<X::O>
    // where
    //     I: ConcurrentIter,
    //     X: Xap<I = I::Item>,
    //     X::O: Send,
    // {
    //     let mut spawned = 0;
    //     let (max_nt, state) = self.nt_state(params, iter.try_get_len());
    //     let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

    //     let (iter, state, results, x) = (&iter, &state, &results_bag, x);
    //     self.pool_mut().scoped_computation(move |s| {
    //         while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
    //             spawned += 1;
    //             <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
    //                 let value = th::next_any::<Self, _, _>(th_idx, state, iter, x);
    //                 results.push(value);
    //             });
    //         }
    //     });

    //     results_bag.into_inner().into_iter().flatten().next()
    // }

    // // provided - option

    // fn option_next<I, X, O>(&mut self, params: Params, iter: I, x: X) -> Option<OptIdx<O>>
    // where
    //     I: ConcurrentIter,
    //     X: Xap<I = I::Item, O = Option<O>>,
    //     O: Send,
    // {
    //     let mut spawned = 0;
    //     let (max_nt, state) = self.nt_state(params, iter.try_get_len());
    //     let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

    //     let (iter, state, results, x) = (&iter, &state, &results_bag, x);
    //     self.pool_mut().scoped_computation(move |s| {
    //         while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
    //             spawned += 1;
    //             <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
    //                 let value = th::option_next::<Self, _, _, _>(th_idx, state, iter, x);
    //                 results.push(value);
    //             });
    //         }
    //     });

    //     OptIdx::first_of(results_bag.into_inner().into_inner())
    // }

    // fn option_next_any<I, X, O>(&mut self, params: Params, iter: I, x: X) -> Option<Opt<O>>
    // where
    //     I: ConcurrentIter,
    //     X: Xap<I = I::Item, O = Option<O>>,
    //     O: Send,
    // {
    //     let mut spawned = 0;
    //     let (max_nt, state) = self.nt_state(params, iter.try_get_len());
    //     let results_bag = ConcurrentBag::with_fixed_capacity(max_nt);

    //     let (iter, state, results, x) = (&iter, &state, &results_bag, x);
    //     self.pool_mut().scoped_computation(move |s| {
    //         while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
    //             spawned += 1;
    //             <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
    //                 let value = th::option_next_any::<Self, _, _, _>(th_idx, state, iter, x);
    //                 results.push(value);
    //             });
    //         }
    //     });

    //     Opt::any_of(results_bag.into_inner().into_inner())
    // }

    // provided - helpers

    fn nt_state(&mut self, params: Params, len: Option<usize>) -> (usize, Self::State) {
        let max_nt = self.pool().max_num_threads_for_computation(params, len);
        let state = self.new_state(params, max_nt, len);
        (max_nt, state)
    }
}
