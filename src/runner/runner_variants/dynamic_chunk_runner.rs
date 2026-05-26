use crate::parameters::{ChunkSize, Params};
use crate::pool::ParThreadPool;
use crate::runner::par_runner::ParRunner;
use crate::runner::runner_variants::fixed_chunk::heuristic;
use core::cmp::{max, min};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct DynChunkRunner<P: ParThreadPool> {
    pool: P,
}

unsafe impl<P: ParThreadPool> Sync for DynChunkRunner<P> {}

impl<P: ParThreadPool> DynChunkRunner<P> {
    pub fn new(pool: P) -> Self {
        Self { pool }
    }
}

pub struct State {
    pub max_num_threads: usize,
    pub initial_len: Option<usize>,
    min_chunk_size: usize,
    max_chunk_size: usize,
    next_chunk_size: AtomicUsize,
    completed_chunks: AtomicUsize,
    assigned_items: AtomicUsize,
}

pub struct ChunkState {
    requested_chunk_size: usize,
}

impl State {
    fn clamp_chunk(&self, chunk_size: usize) -> usize {
        min(
            max(chunk_size, self.min_chunk_size),
            max(self.max_chunk_size, self.min_chunk_size),
        )
    }

    fn set_next_chunk_size_relaxed(&self, new_value: usize) {
        let new_value = self.clamp_chunk(new_value);

        let mut current = self.next_chunk_size.load(Ordering::Relaxed);
        loop {
            let merged = match new_value < current {
                true => new_value,
                false => (current + new_value) / 2,
            };

            match self.next_chunk_size.compare_exchange_weak(
                current,
                merged,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(observed) => current = observed,
            }
        }
    }
}

impl<P: ParThreadPool> ParRunner for DynChunkRunner<P> {
    type Pool = P;

    type State = State;

    type ChunkState = ChunkState;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.pool
    }

    fn with_pool<Q: ParThreadPool>(
        self,
        pool: Q,
    ) -> impl ParRunner<State = Self::State, ChunkState = Self::ChunkState, Pool = Q> {
        DynChunkRunner::new(pool)
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        (spawned < state.max_num_threads).then_some(spawned)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State {
        let base_chunk_size =
            heuristic::compute_chunk_size(params.chunk_size, initial_len, max_num_threads);

        let min_chunk_size = match params.chunk_size {
            ChunkSize::Auto => 1,
            ChunkSize::Min(c) | ChunkSize::Exact(c) => c.into(),
        };

        let mut max_chunk_size = match params.chunk_size {
            ChunkSize::Exact(c) => c.into(),
            ChunkSize::Min(c) => max(c.into(), base_chunk_size.saturating_mul(4)),
            ChunkSize::Auto => max(base_chunk_size.saturating_mul(8), 1),
        };

        if let Some(initial_len) = initial_len {
            let per_thread = max(initial_len / max_num_threads.max(1), 1);
            max_chunk_size = min(max_chunk_size, max(per_thread, min_chunk_size));
        }

        let next_chunk_size = min(max(base_chunk_size, min_chunk_size), max_chunk_size);

        State {
            max_num_threads,
            initial_len,
            min_chunk_size,
            max_chunk_size,
            next_chunk_size: AtomicUsize::new(next_chunk_size),
            completed_chunks: AtomicUsize::new(0),
            assigned_items: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    fn begin_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn next_chunk_size(state: &Self::State, size_hint: (usize, Option<usize>)) -> usize {
        let current = state.next_chunk_size.load(Ordering::Relaxed);

        let hinted_remaining = size_hint.1.unwrap_or(size_hint.0);
        if hinted_remaining == 0 {
            return 1;
        }

        if hinted_remaining <= state.max_num_threads {
            return 1;
        }

        let target_parallel_rounds = state.max_num_threads.saturating_mul(4).max(1);
        let target = max(
            hinted_remaining / target_parallel_rounds,
            state.min_chunk_size,
        );
        let target = min(target, state.max_chunk_size);

        // Keep changes smooth to avoid oscillation under heterogeneous workloads.
        let smoothed = (current.saturating_mul(3) + target) / 4;
        let next = state.clamp_chunk(smoothed);
        state.set_next_chunk_size_relaxed(next);

        min(next, hinted_remaining)
    }

    #[inline(always)]
    fn begin_chunk(_: usize, chunk_size: usize) -> Self::ChunkState {
        ChunkState {
            requested_chunk_size: chunk_size,
        }
    }

    #[inline(always)]
    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState) {
        let completed = state.completed_chunks.fetch_add(1, Ordering::Relaxed) + 1;
        let assigned = state
            .assigned_items
            .fetch_add(chunk_state.requested_chunk_size, Ordering::Relaxed)
            + chunk_state.requested_chunk_size;

        if let Some(total) = state.initial_len {
            let half = total / 2;
            let eighth = total / 8;
            let remaining = total.saturating_sub(assigned);

            if remaining <= eighth {
                state.set_next_chunk_size_relaxed(state.min_chunk_size);
            } else if remaining <= half {
                state.set_next_chunk_size_relaxed(max(chunk_state.requested_chunk_size / 2, 1));
            }
        } else {
            // For unknown-length iterators, progressively lower chunk size after each
            // wave of work to improve late-stage balancing.
            let wave = state.max_num_threads.max(1) * 8;
            if completed % wave == 0 {
                let reduced = max(chunk_state.requested_chunk_size.saturating_mul(7) / 8, 1);
                state.set_next_chunk_size_relaxed(reduced);
            }
        }
    }

    #[inline(always)]
    fn complete_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn complete_computation(_: Self::State) {}
}
