use crate::parameters::Params;
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
/// 2. **Queue-depth-gated thread spawning** — before spawning thread `t`, the runner checks that
///    the iterator's current queue lower bound satisfies
///    `queue_lower_bound >= t * min_items_per_thread`.  
///    This prevents spawning 32 threads when the queue only contains 50 items; new threads are
///    spawned only when enough work is visible to keep them busy.  Since threads start executing
///    and extending the queue while the spawning loop runs, additional threads can still be
///    admitted as the queue grows.
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
        queue_lower_bound: usize,
    ) -> Option<usize> {
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
        // For unknown-length (recursive) iterators: only spawn thread `spawned` if the observable
        // queue already holds enough items to keep it busy.
        (queue_lower_bound >= spawned * state.min_items_per_thread).then_some(spawned)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State {
        let chunk_size = match initial_len {
            // Known length: delegate to the standard heuristic (same as FixedChunkRunner).
            Some(_) => {
                fixed_heuristic::compute_chunk_size(params.chunk_size, initial_len, max_num_threads)
            }
            // Unknown length (recursive): use the recursive-specific heuristic.
            None => heuristic::compute_chunk_size(params.chunk_size, max_num_threads),
        };
        let min_items_per_thread = heuristic::compute_min_items_per_thread(chunk_size);
        State {
            max_num_threads,
            initial_len,
            chunk_size,
            min_items_per_thread,
        }
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
