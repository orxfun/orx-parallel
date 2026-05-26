use crate::parameters::{ChunkSize, Params};
use crate::pool::ParThreadPool;
use crate::runner::par_runner::ParRunner;
use core::cmp::min;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

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
    min_chunk_size: usize,
    fixed_chunk_size: Option<usize>,
    target_chunk_time_ns: u64,
    next_chunk_size: AtomicUsize,
    sample_count: AtomicUsize,
    avg_ns_per_item: AtomicU64,
    avg_abs_deviation_ns_per_item: AtomicU64,
    conservative_runs: AtomicUsize,
}

pub struct ChunkState {
    requested_chunk_size: usize,
    started_at: Instant,
}

const TARGET_CHUNK_TIME_NS: u64 = 1_000_000;
const MIN_STABLE_SAMPLES: usize = 5;
const LOW_VARIABILITY_PCT: u64 = 15;
const HIGH_VARIABILITY_PCT: u64 = 40;
const SLOWDOWN_FACTOR_PCT: u64 = 125;

impl State {
    fn ratio_to_pct(numerator: u64, denominator: u64) -> u64 {
        if denominator == 0 {
            0
        } else {
            numerator.saturating_mul(100) / denominator
        }
    }

    fn ewma(previous: u64, sample: u64, numerator: u64, denominator: u64) -> u64 {
        match previous {
            0 => sample,
            n => (n.saturating_mul(numerator) + sample) / denominator,
        }
    }

    fn take_conservative_run(&self) -> bool {
        self.conservative_runs
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_sub(1)
            })
            .is_ok()
    }

    fn set_next_chunk_size(&self, chunk_size: usize) {
        self.next_chunk_size
            .store(chunk_size.max(self.min_chunk_size), Ordering::Relaxed);
    }

    fn min_chunk_for_remaining(&self, remaining: usize) -> usize {
        min(self.min_chunk_size, remaining.max(1))
    }

    fn adapt_from_sample(&self, elapsed_ns: u64, chunk_size: usize) {
        let items = chunk_size.max(1) as u64;
        let sample_ns_per_item = elapsed_ns / items;

        let previous_avg = self.avg_ns_per_item.load(Ordering::Relaxed);
        let updated_avg = Self::ewma(previous_avg, sample_ns_per_item, 7, 8);
        self.avg_ns_per_item.store(updated_avg, Ordering::Relaxed);

        let deviation = updated_avg.abs_diff(sample_ns_per_item);
        let previous_dev = self.avg_abs_deviation_ns_per_item.load(Ordering::Relaxed);
        let updated_dev = Self::ewma(previous_dev, deviation, 3, 4);
        self.avg_abs_deviation_ns_per_item
            .store(updated_dev, Ordering::Relaxed);

        self.sample_count.fetch_add(1, Ordering::Relaxed);

        if updated_avg == 0 {
            return;
        }

        let variability = Self::ratio_to_pct(updated_dev, updated_avg);
        let slowdown = Self::ratio_to_pct(sample_ns_per_item, updated_avg);

        if variability >= HIGH_VARIABILITY_PCT || slowdown >= SLOWDOWN_FACTOR_PCT {
            let cool_down = 4 + (variability as usize / 10);
            self.conservative_runs
                .store(cool_down.max(2), Ordering::Relaxed);
            self.set_next_chunk_size(self.min_chunk_size);
            return;
        }

        if variability <= LOW_VARIABILITY_PCT
            && self.sample_count.load(Ordering::Relaxed) >= MIN_STABLE_SAMPLES
        {
            let current = self
                .next_chunk_size
                .load(Ordering::Relaxed)
                .max(self.min_chunk_size);
            let ramp = current
                .saturating_add(current / 2)
                .max(current.saturating_mul(2));
            self.set_next_chunk_size(ramp);
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
        _initial_len: Option<usize>,
    ) -> Self::State {
        let min_chunk_size = match params.chunk_size {
            ChunkSize::Auto => 1,
            ChunkSize::Min(c) | ChunkSize::Exact(c) => c.into(),
        };

        let fixed_chunk_size = match params.chunk_size {
            ChunkSize::Exact(c) => Some(c.into()),
            _ => None,
        };

        let initial_chunk_size = fixed_chunk_size.unwrap_or(min_chunk_size);

        State {
            max_num_threads,
            min_chunk_size,
            fixed_chunk_size,
            target_chunk_time_ns: TARGET_CHUNK_TIME_NS,
            next_chunk_size: AtomicUsize::new(initial_chunk_size),
            sample_count: AtomicUsize::new(0),
            avg_ns_per_item: AtomicU64::new(0),
            avg_abs_deviation_ns_per_item: AtomicU64::new(0),
            conservative_runs: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    fn begin_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn next_chunk_size(state: &Self::State, size_hint: (usize, Option<usize>)) -> usize {
        if let Some(fixed_chunk_size) = state.fixed_chunk_size {
            let remaining = size_hint.1.unwrap_or(size_hint.0).max(1);
            return min(fixed_chunk_size, remaining);
        }

        let remaining = size_hint.1.unwrap_or(size_hint.0);
        if remaining == 0 {
            return 1;
        }

        if state.sample_count.load(Ordering::Relaxed) == 0 {
            return state.min_chunk_for_remaining(remaining);
        }

        if state.sample_count.load(Ordering::Relaxed) < MIN_STABLE_SAMPLES {
            return state.min_chunk_for_remaining(remaining);
        }

        if state.take_conservative_run() {
            return state.min_chunk_for_remaining(remaining);
        }

        let avg = state.avg_ns_per_item.load(Ordering::Relaxed);
        let dev = state.avg_abs_deviation_ns_per_item.load(Ordering::Relaxed);
        if avg == 0 {
            return state.min_chunk_for_remaining(remaining);
        }

        let variability = State::ratio_to_pct(dev, avg);
        if variability >= HIGH_VARIABILITY_PCT {
            return state.min_chunk_for_remaining(remaining);
        }

        let min_chunk = state.min_chunk_for_remaining(remaining);
        let current = state.next_chunk_size.load(Ordering::Relaxed).max(min_chunk);
        let target_from_time =
            (state.target_chunk_time_ns / avg.max(1)).max(min_chunk as u64) as usize;
        let target_from_balance =
            (remaining / state.max_num_threads.max(1).saturating_mul(8)).max(min_chunk);

        let growth_factor = if variability <= LOW_VARIABILITY_PCT {
            2
        } else {
            3
        };
        let growth_divisor = if variability <= LOW_VARIABILITY_PCT {
            1
        } else {
            2
        };
        let grown = current.saturating_mul(growth_factor) / growth_divisor;

        let mut desired = grown.max(target_from_time);
        desired = desired.min(target_from_balance.max(min_chunk));
        desired = desired.min(remaining);
        desired = desired.max(min_chunk);

        let max_step = current.saturating_add((current.max(2)) / 2);
        desired = desired.min(max_step.max(min_chunk));

        state.set_next_chunk_size(desired);
        desired
    }

    #[inline(always)]
    fn begin_chunk(_: usize, chunk_size: usize) -> Self::ChunkState {
        ChunkState {
            requested_chunk_size: chunk_size,
            started_at: Instant::now(),
        }
    }

    #[inline(always)]
    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState) {
        let elapsed_ns = chunk_state
            .started_at
            .elapsed()
            .as_nanos()
            .min(u64::MAX as u128) as u64;
        state.adapt_from_sample(elapsed_ns, chunk_state.requested_chunk_size);
    }

    #[inline(always)]
    fn complete_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn complete_computation(_: Self::State) {}
}
