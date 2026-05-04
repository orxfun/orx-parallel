use crate::parameters::{NumThreads, Params};
use crate::pool::ParThreadPool;
use crate::runner::par_runner::ParRunner;
use crate::runner::runner_variants::{
    fixed_chunk::heuristic as fixed_heuristic,
    recursive_chunk::{heuristic, state::State},
};

/// A [`ParRunner`] variant tuned for parallel iterators with **unknown or dynamically-growing
/// length** — most notably `into_par_recursive`.
///
/// ## Problem with the default runner on recursive workloads
///
/// The default [`FixedChunkRunner`](crate::runner::runner_variants::FixedChunkRunner) spawns all
/// available threads immediately and uses `chunk_size = 1` for unknown-length iterators.  
/// For recursive workloads the iterator starts with a small initial queue (e.g. 50 root nodes).
/// Because `std::thread::scope` creates OS threads sequentially, each thread creation takes
/// ~100 µs.  By the time the last of 32 threads is spawned (~3 ms later), the first few threads
/// have already pulled thousands of items from the queue, leading to severe work imbalance and
/// high wall-time variance.
///
/// ## What this runner does differently
///
/// 1. **Larger default chunk size** — uses `chunk_size = 64` instead of `1` for unknown-length
///    iterators.  This reduces atomic queue contention (64× fewer queue operations) and gives
///    each thread a meaningful batch of work immediately.
///
/// 2. **Deterministic spawning with an auto thread cap** — for unknown-length recursive
///    iterators, this runner spawns threads deterministically up to a capped thread count
///    (`MAX_RECURSIVE_AUTO_THREADS` when `num_threads` is `Auto`).
///    This avoids brittle queue-length gating during sequential spawning, where the first worker
///    can briefly drain/reserve the initial frontier and make the queue appear empty.
pub struct RecursiveChunkRunner<P: ParThreadPool> {
    pool: P,
}

unsafe impl<P: ParThreadPool> Sync for RecursiveChunkRunner<P> {}

impl<P: ParThreadPool> RecursiveChunkRunner<P> {
    pub fn new(pool: P) -> Self {
        Self { pool }
    }
}

impl<P: ParThreadPool> ParRunner for RecursiveChunkRunner<P> {
    type Pool = P;
    type State = State;
    type ChunkState = ();

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.pool
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        (spawned < state.max_num_threads).then_some(spawned)
    }

    fn do_spawn_new_with_queue_len(
        spawned: usize,
        state: &Self::State,
        _queue_lower_bound: usize,
    ) -> Option<usize> {
        std::println!(
            "{spawned} => {_queue_lower_bound} ==> {:?}",
            state.initial_len
        );
        if spawned >= state.max_num_threads {
            return None;
        }
        if spawned == 0 {
            // Always spawn at least one thread.
            return Some(0);
        }
        // For iterators whose length is known up-front, fall back to always-spawn behaviour:
        // the FixedChunkRunner heuristic already sized chunk_size appropriately.
        if state.initial_len.is_some() {
            return (spawned < state.max_num_threads).then_some(spawned);
        }
        // For unknown-length (recursive) iterators, queue lower bound is too transient during
        // the sequential spawning phase: the first worker can reserve/drain the initial frontier
        // before later spawn checks observe it, which falsely blocks parallelism.
        // Therefore, this runner spawns up to the selected thread cap deterministically.
        (spawned < state.max_num_threads).then_some(spawned)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        size_hint: (usize, Option<usize>),
    ) -> Self::State {
        // let max_num_threads = match (initial_len, params.num_threads) {
        //     (None, NumThreads::Auto) => max_num_threads.min(heuristic::MAX_RECURSIVE_AUTO_THREADS),
        //     _ => max_num_threads,
        // };

        // let chunk_size = match initial_len {
        //     // Known length: delegate to the standard heuristic (same as FixedChunkRunner).
        //     Some(_) => {
        //         fixed_heuristic::compute_chunk_size(params.chunk_size, initial_len, max_num_threads)
        //     }
        //     // Unknown length (recursive): use the recursive-specific heuristic.
        //     None => heuristic::compute_chunk_size(params.chunk_size, max_num_threads),
        // };
        // State {
        //     max_num_threads,
        //     initial_len,
        //     chunk_size,
        // }

        todo!()
    }

    #[inline(always)]
    fn begin_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn next_chunk_size(state: &Self::State, _: Option<usize>) -> usize {
        state.chunk_size
    }

    #[inline(always)]
    fn begin_chunk(_: usize, _: usize) -> Self::ChunkState {}

    #[inline(always)]
    fn complete_chunk_non_empty(_: &Self::State, _: Self::ChunkState) {}

    #[inline(always)]
    fn complete_chunk_empty(_: &Self::State, _: Self::ChunkState) {}

    #[inline(always)]
    fn complete_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn complete_computation(_: Self::State) {}
}
