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

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Explore,
    Bulk,
    Adaptive,
}

pub struct State {
    pub max_num_threads: usize,
    min_chunk_size: usize,
    fixed_chunk_size: Option<usize>,
    mode: AtomicUsize,
    next_chunk_size: AtomicUsize,
    sample_count: AtomicUsize,
    avg_ns_per_item: AtomicU64,
    avg_abs_deviation_ns_per_item: AtomicU64,
    conservative_runs: AtomicUsize,
    initial_len: Option<usize>,
    probe_idx: AtomicUsize,
    measurements_in_phase: AtomicUsize,
}

pub struct ChunkState {
    requested_chunk_size: usize,
    started_at: Instant,
}

const PROBE_SIZES: &[usize] = &[1, 4, 16, 64, 256];
const NUM_PROBES: usize = 5;
const LOW_VARIABILITY_PCT: u64 = 20;
const HIGH_VARIABILITY_PCT: u64 = 50;
const SLOWDOWN_FACTOR_PCT: u64 = 135;

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

    fn mode(&self) -> Mode {
        match self.mode.load(Ordering::Relaxed) {
            0 => Mode::Explore,
            1 => Mode::Bulk,
            _ => Mode::Adaptive,
        }
    }

    fn set_mode(&self, m: Mode) {
        self.mode.store(
            match m {
                Mode::Explore => 0,
                Mode::Bulk => 1,
                Mode::Adaptive => 2,
            },
            Ordering::Relaxed,
        );
    }

    fn set_next_chunk_size(&self, chunk_size: usize) {
        self.next_chunk_size
            .store(chunk_size.max(self.min_chunk_size), Ordering::Relaxed);
    }

    fn min_chunk_for_remaining(&self, remaining: usize) -> usize {
        min(self.min_chunk_size, remaining.max(1))
    }

    fn should_measure_this_chunk(&self) -> bool {
        match self.mode() {
            Mode::Explore => true,
            Mode::Bulk => {
                let samples = self.sample_count.load(Ordering::Relaxed);
                samples < 10 || (samples % 50) == 0
            }
            Mode::Adaptive => true,
        }
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

        match self.mode() {
            Mode::Explore => {
                self.measurements_in_phase.fetch_add(1, Ordering::Relaxed);
                let meas = self.measurements_in_phase.load(Ordering::Relaxed);
                if meas >= NUM_PROBES {
                    self.measurements_in_phase.store(0, Ordering::Relaxed);
                    if variability <= LOW_VARIABILITY_PCT * 2 {
                        self.set_mode(Mode::Bulk);
                        let bulk_size = self.compute_bulk_chunk_size();
                        self.set_next_chunk_size(bulk_size);
                    } else {
                        self.set_mode(Mode::Adaptive);
                    }
                    self.probe_idx.store(0, Ordering::Relaxed);
                }
            }
            Mode::Bulk => {
                if slowdown >= SLOWDOWN_FACTOR_PCT {
                    self.set_mode(Mode::Adaptive);
                }
            }
            Mode::Adaptive => {
                if variability >= HIGH_VARIABILITY_PCT || slowdown >= SLOWDOWN_FACTOR_PCT {
                    let cool_down = 2 + (variability as usize / 20);
                    self.conservative_runs
                        .store(cool_down.max(1), Ordering::Relaxed);
                    self.set_next_chunk_size(self.min_chunk_size);
                } else if variability <= LOW_VARIABILITY_PCT {
                    let current = self
                        .next_chunk_size
                        .load(Ordering::Relaxed)
                        .max(self.min_chunk_size);
                    let ramp = current
                        .saturating_add(current / 3)
                        .max(current.saturating_mul(2));
                    self.set_next_chunk_size(ramp);
                }
            }
        }
    }

    fn compute_bulk_chunk_size(&self) -> usize {
        match self.initial_len {
            Some(total) => {
                let per_thread = total / self.max_num_threads.max(1);
                per_thread / 4
            }
            None => 256,
        }
        .max(self.min_chunk_size)
    }

    fn take_conservative_run(&self) -> bool {
        self.conservative_runs
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_sub(1)
            })
            .is_ok()
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
        let min_chunk_size = match params.chunk_size {
            ChunkSize::Auto => 1,
            ChunkSize::Min(c) | ChunkSize::Exact(c) => c.into(),
        };

        let fixed_chunk_size = match params.chunk_size {
            ChunkSize::Exact(c) => Some(c.into()),
            _ => None,
        };

        let initial_chunk_size = fixed_chunk_size.unwrap_or(1);

        State {
            max_num_threads,
            min_chunk_size,
            fixed_chunk_size,
            mode: AtomicUsize::new(0),
            next_chunk_size: AtomicUsize::new(initial_chunk_size),
            sample_count: AtomicUsize::new(0),
            avg_ns_per_item: AtomicU64::new(0),
            avg_abs_deviation_ns_per_item: AtomicU64::new(0),
            conservative_runs: AtomicUsize::new(0),
            initial_len,
            probe_idx: AtomicUsize::new(0),
            measurements_in_phase: AtomicUsize::new(0),
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

        match state.mode() {
            Mode::Explore => {
                let probe_idx = state.probe_idx.load(Ordering::Relaxed);
                if probe_idx < NUM_PROBES {
                    let chunk_size = PROBE_SIZES[probe_idx];
                    state.probe_idx.fetch_add(1, Ordering::Relaxed);
                    return min(chunk_size, remaining);
                } else {
                    return state.min_chunk_for_remaining(remaining);
                }
            }
            Mode::Bulk => {
                let current = state.next_chunk_size.load(Ordering::Relaxed);
                return min(current, remaining);
            }
            Mode::Adaptive => {
                if state.take_conservative_run() {
                    return state.min_chunk_for_remaining(remaining);
                }

                let current = state
                    .next_chunk_size
                    .load(Ordering::Relaxed)
                    .max(state.min_chunk_size);
                return min(current, remaining);
            }
        }
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
        if state.should_measure_this_chunk() {
            let elapsed_ns = chunk_state
                .started_at
                .elapsed()
                .as_nanos()
                .min(u64::MAX as u128) as u64;
            state.adapt_from_sample(elapsed_ns, chunk_state.requested_chunk_size);
        } else {
            state.sample_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[inline(always)]
    fn complete_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn complete_computation(_: Self::State) {}
}
