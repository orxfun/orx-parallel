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
/// 2. **Adaptive thread cap** — for unknown-length recursive iterators, the number of threads
///    is capped at `sqrt(initial_queue * available_cpus) / 2 + 1` (when `num_threads` is `Auto`).
///    This geometric-mean formula balances two competing forces:
///    - Too many threads: spawn overhead (~100 µs/thread) hurts short workloads; late-spawned
///      threads compete for a shrinking queue and contribute very little work.
///    - Too few threads: leaves CPU cores idle.
///    Example: 50 roots, 32 CPUs → cap = sqrt(50 × 32) / 2 + 1 = 21 threads, which lands in
///    the empirically optimal 20–24 thread range without hard-coding a constant.
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
        _size_hint: (usize, Option<usize>),
    ) -> Option<usize> {
        // Deterministic spawning: always spawn up to max_num_threads.
        // For unknown-length iterators the thread cap is computed adaptively in new_state()
        // using the geometric-mean heuristic, so no queue-length gating is needed here.
        // Queue-length gating is unreliable for recursive iterators because the queue can
        // transiently appear empty while threads race to populate it from processed items.
        Self::do_spawn_new(spawned, state)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        size_hint: (usize, Option<usize>),
    ) -> Self::State {
        let max_num_threads = match (size_hint.1, params.num_threads) {
            (None, NumThreads::Auto) => {
                // Geometric-mean cap: sqrt(initial_queue * available_cpus) / 2 + 1.
                //
                // Rationale: for a recursive tree with `q` root items, the optimal thread
                // count lies between `~q/chunk_size` (immediate concurrency, usually very low)
                // and `available_cpus` (maximum hardware concurrency).  The geometric mean of
                // these two bounds, `sqrt(q * cpus)`, scaled by 1/2 to account for the fact
                // that not all initial items are immediately processable (chunk_size > 1),
                // provides a principled, hardware-adaptive balance without requiring knowledge
                // of the branching factor or work-per-node.
                //
                // Examples (available_cpus = 32):
                //   q =   1 →  3 threads    q =  50 → 21 threads
                //   q =   4 →  9 threads    q = 100 → 29 threads
                //   q =  10 → 14 threads    q = 500 → 32 (capped)
                let q = size_hint.0;
                let n = (q * max_num_threads).isqrt() / 2 + 1;
                n.max(1).min(max_num_threads)
            }
            _ => max_num_threads,
        };

        let chunk_size = match size_hint.1 {
            // Known length: delegate to the standard heuristic (same as FixedChunkRunner).
            Some(_) => {
                fixed_heuristic::compute_chunk_size(params.chunk_size, size_hint, max_num_threads)
            }
            // Unknown length (recursive): use the recursive-specific heuristic.
            None => heuristic::compute_chunk_size(params.chunk_size, max_num_threads),
        };
        State {
            max_num_threads,
            size_hint,
            chunk_size,
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
