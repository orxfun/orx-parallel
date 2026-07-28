use crate::runner::runner_variants::run_b::chunk_state::ChunkState;
use crate::runner::runner_variants::run_b::constants::*;
use crate::runner::runner_variants::run_b::mode::{AtomicMode, Mode};
use core::cmp::{max, min};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

pub struct State {
    pub max_num_threads: usize,
    pub min_chunk_size: usize,
    pub fixed_chunk_size: Option<usize>,
    initial_len: Option<usize>,
    explore_started_at: Instant,
    mode: AtomicMode,
    chosen_chunk_size: AtomicUsize,
    explored_tasks: AtomicUsize,
    avg_ns_per_item: AtomicU64,
    avg_abs_deviation_ns_per_item: AtomicU64,
    prev_avg_ns_per_item: AtomicU64,
    converged_samples: AtomicUsize,
    min_samples: usize,
}

impl State {
    pub(super) fn new(
        max_num_threads: usize,
        min_chunk_size: usize,
        fixed_chunk_size: Option<usize>,
        initial_len: Option<usize>,
    ) -> Self {
        debug_assert!(max_num_threads > 0);

        let min_samples = max(
            EXPLORATION_MIN_SAMPLES_BASE,
            EXPLORATION_SAMPLES_PER_THREAD * max_num_threads,
        );

        Self {
            max_num_threads,
            min_chunk_size,
            fixed_chunk_size,
            initial_len,
            explore_started_at: std::time::Instant::now(),
            mode: AtomicMode::new_explore(),
            chosen_chunk_size: AtomicUsize::new(fixed_chunk_size.unwrap_or(0)),
            explored_tasks: AtomicUsize::new(0),
            avg_ns_per_item: AtomicU64::new(0),
            avg_abs_deviation_ns_per_item: AtomicU64::new(0),
            prev_avg_ns_per_item: AtomicU64::new(0),
            converged_samples: AtomicUsize::new(0),
            min_samples,
        }
    }

    #[inline(always)]
    pub(super) fn mode(&self) -> Mode {
        self.mode.mode()
    }

    #[inline(always)]
    pub(super) fn complete_exploration(&self) {
        self.mode.complete_exploration();
    }

    pub(super) fn record_chunk(&self, chunk_state: ChunkState) {
        let elapsed_ns = chunk_state.elapsed_ns();
        let items = chunk_state.requested_chunk_size.max(1);
        let sample_ns_per_item = elapsed_ns / items as u64;

        let previous_avg = self.avg_ns_per_item.load(Ordering::Relaxed);
        let updated_avg = EWMA_PARAMS_AVG.ewma(previous_avg, sample_ns_per_item);
        self.avg_ns_per_item.store(updated_avg, Ordering::Relaxed);

        let deviation = updated_avg.abs_diff(sample_ns_per_item);
        let previous_dev = self.avg_abs_deviation_ns_per_item.load(Ordering::Relaxed);
        let updated_dev = EWMA_PARAMS_DEV.ewma(previous_dev, deviation);
        self.avg_abs_deviation_ns_per_item
            .store(updated_dev, Ordering::Relaxed);

        self.explored_tasks.fetch_add(items, Ordering::Relaxed);

        // Track convergence: if average is stable, increment counter
        // average stable == change < prev_avg / CONVERGENCE_STABILITY_DIVISOR
        // (i.e., percentage change < 1/CONVERGENCE_STABILITY_DIVISOR ≈ 2%)
        let prev_avg = self.prev_avg_ns_per_item.load(Ordering::Relaxed);
        if prev_avg > 0 {
            let change = updated_avg.abs_diff(prev_avg);
            let threshold = prev_avg / CONVERGENCE_STABILITY_DIVISOR;
            let stable = change < threshold;
            match stable {
                true => _ = self.converged_samples.fetch_add(1, Ordering::Relaxed),
                false => self.converged_samples.store(0, Ordering::Relaxed), // reset
            }
        }
        self.prev_avg_ns_per_item
            .store(updated_avg, Ordering::Relaxed);
    }

    pub(super) fn should_stop_exploration(&self) -> bool {
        let explored = self.explored_tasks.load(Ordering::Relaxed);
        let explored_x100 = explored.saturating_mul(100);
        let fraction_capped = match self.initial_len {
            Some(total) if total > 0 => explored_x100 >= total * EXPLORATION_CAP_PCT,
            _ => false,
        };
        if fraction_capped {
            return true;
        }

        if explored < self.min_samples {
            return false;
        }

        let fraction_reached = match self.initial_len {
            Some(total) if total > 0 => explored_x100 >= total * EXPLORATION_TARGET_PCT,
            _ => false,
        };

        if fraction_reached {
            return true;
        }

        // Convergence-based stopping: if average has been stable for N samples,
        // and we've explored enough, stop exploring
        let converged = self.converged_samples.load(Ordering::Relaxed);
        if converged >= CONVERGENCE_THRESHOLD && explored >= CONVERGENCE_MIN_SAMPLES {
            return true;
        }

        let avg_ns = self.avg_ns_per_item.load(Ordering::Relaxed);
        let elapsed_ms = self.explore_started_at.elapsed().as_millis();

        // Early exit for tiny work: if per-item work is extremely small,
        // exploration overhead dominates. Stop after minimal sampling.
        if avg_ns > 0 && avg_ns < TINY_WORK_THRESHOLD_NS && explored >= TINY_WORK_MIN_SAMPLES {
            return true;
        }

        elapsed_ms >= EXPLORATION_MIN_MS
    }

    pub(super) fn selected_chunk_size(&self, size_hint: (usize, Option<usize>)) -> usize {
        match self.chosen_chunk_size.load(Ordering::Relaxed) {
            0 => {
                let chunk_size = self.choose_fixed_chunk_size(size_hint);
                self.chosen_chunk_size.store(chunk_size, Ordering::Relaxed);
                chunk_size
            }
            chunk_size => chunk_size,
        }
    }

    fn variability_pct(&self) -> u64 {
        match self.avg_ns_per_item.load(Ordering::Relaxed) {
            0 => 0,
            avg => self
                .avg_abs_deviation_ns_per_item
                .load(Ordering::Relaxed)
                .saturating_mul(100)
                .checked_div(avg)
                .unwrap_or(0),
        }
    }

    fn choose_fixed_chunk_size(&self, size_hint: (usize, Option<usize>)) -> usize {
        let avg_ns = self.avg_ns_per_item.load(Ordering::Relaxed);
        if avg_ns == 0 || avg_ns >= HEAVY_WORK_NS_THRESHOLD {
            return self.min_chunk_size;
        }

        let variability_pct = self.variability_pct();
        if variability_pct >= HIGH_VARIABILITY_PCT_THRESHOLD {
            return self.min_chunk_size;
        }

        let amortized_chunk_size = max(
            AMORTIZED_OVERHEAD_NS.div_ceil(avg_ns) as usize,
            self.min_chunk_size,
        );

        let waves = balance_waves(variability_pct);
        let balanced_chunk_size = match size_hint.1 {
            Some(remaining) => remaining / (self.max_num_threads * waves),
            _ => fallback_balance_bound(variability_pct),
        };

        max(
            min3(balanced_chunk_size, amortized_chunk_size, MAX_CHUNK_SIZE),
            self.min_chunk_size,
        )
    }
}

#[inline(always)]
fn min3(a: usize, b: usize, c: usize) -> usize {
    min(min(a, b), c)
}
