use crate::parameters::{NumThreads, Params};
use crate::runner::thread_computations as th;
use crate::{pool::ParThreadPool, xap::Xap};
use core::num::NonZeroUsize;
use orx_concurrent_bag::ConcurrentBag;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

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

    /// Returns the next chunk size to be pulled from the input with `remaining` length
    /// for the current `state`.
    fn next_chunk_size(state: &Self::State, remaining: Option<usize>) -> usize;

    // required - state updates

    /// Creates an initial state for a new parallel computation.
    fn new_state(&mut self) -> Self::State;

    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState;

    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState);

    fn complete_computation(state: Self::State);

    // provided

    fn next<I, X>(&mut self, params: Params, iter: I, x: X)
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item>,
        X::O: Send,
    {
        let state = self.new_state();
        let mut spawned = 0;
        let results = self.thread_results(params, iter.try_get_len());

        let (iter, state, results, x) = (&iter, &state, &results, x);
        self.pool_mut().scoped_computation(move |s| {
            while let Some(th_idx) = Self::do_spawn_new(spawned, state) {
                spawned += 1;
                <Self::Pool as ParThreadPool>::run_in_scope(&s, move || {
                    let value = th::next::<Self, _, _>(th_idx, state, iter, x);
                    results.push(value);
                });
            }
        });
    }

    // provided - pool

    /// Returns the maximum number of threads that can be used for the computation defined by
    /// the `params` and input `iter_len`.
    fn max_num_threads(&self, params: Params, iter_len: Option<usize>) -> usize {
        let pool = self.pool().max_num_threads();

        let env = crate::pool::max_num_threads_by_env_variable().unwrap_or(NonZeroUsize::MAX);

        let req = match (iter_len, params.num_threads) {
            (Some(len), NumThreads::Auto) => NonZeroUsize::new(len.max(1)).expect(">0"),
            (Some(len), NumThreads::Max(nt)) => NonZeroUsize::new(len.max(1)).expect(">0").min(nt),
            (None, NumThreads::Auto) => NonZeroUsize::MAX,
            (None, NumThreads::Max(nt)) => nt,
        };

        req.min(pool.min(env)).into()
    }

    fn thread_results<T>(
        &self,
        params: Params,
        iter_len: Option<usize>,
    ) -> ConcurrentBag<T, FixedVec<T>> {
        ConcurrentBag::with_fixed_capacity(self.max_num_threads(params, iter_len))
    }
}
