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
}

impl State {
    pub(super) fn new(
        max_num_threads: usize,
        min_chunk_size: usize,
        fixed_chunk_size: Option<usize>,
        initial_len: Option<usize>,
    ) -> Self {
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
        let avg_ns = self.avg_ns_per_item.load(Ordering::Relaxed);
        let min_samples = max(128, 8 * self.max_num_threads.max(1));
        let elapsed_ms = self.explore_started_at.elapsed().as_millis();

        // Early exit for tiny work: if per-item work is extremely small,
        // exploration overhead dominates. Stop after minimal sampling.
        if avg_ns > 0 && avg_ns < TINY_WORK_THRESHOLD_NS && explored >= TINY_WORK_MIN_SAMPLES {
            return true;
        }

        // Convergence-based stopping: if average has been stable for 5 samples,
        // and we've explored enough, stop exploring
        let converged = self.converged_samples.load(Ordering::Relaxed);
        if converged >= CONVERGENCE_THRESHOLD && explored >= CONVERGENCE_MIN_SAMPLES {
            return true;
        }

        let fraction_reached = match self.initial_len {
            Some(total) if total > 0 => {
                explored.saturating_mul(100) >= total * EXPLORATION_TARGET_PCT
            }
            _ => false,
        };

        let fraction_capped = match self.initial_len {
            Some(total) if total > 0 => explored.saturating_mul(100) >= total * EXPLORATION_CAP_PCT,
            _ => false,
        };

        fraction_capped
            || (explored >= min_samples && (elapsed_ms >= EXPLORATION_MIN_MS || fraction_reached))
    }

    fn variability_pct(&self) -> u64 {
        let avg = self.avg_ns_per_item.load(Ordering::Relaxed);
        if avg == 0 {
            return 0;
        }

        self.avg_abs_deviation_ns_per_item
            .load(Ordering::Relaxed)
            .saturating_mul(100)
            .checked_div(avg)
            .unwrap_or(0)
    }

    fn fallback_balance_bound(variability_pct: u64) -> usize {
        match variability_pct {
            v if v < 25 => 128,
            v if v < 75 => 64,
            v if v < 150 => 16,
            _ => 4,
        }
    }

    fn choose_fixed_chunk_size(&self, size_hint: (usize, Option<usize>)) -> usize {
        let avg_ns = self.avg_ns_per_item.load(Ordering::Relaxed);
        if avg_ns == 0 {
            return self.min_chunk_size;
        }

        let variability_pct = self.variability_pct();
        if avg_ns >= 200 * OVERHEAD_NS_PER_CHUNK || variability_pct >= 150 {
            return self.min_chunk_size;
        }

        let amortized = OVERHEAD_AMORTIZATION_FACTOR.saturating_mul(OVERHEAD_NS_PER_CHUNK);
        let mut c_over = amortized.div_ceil(avg_ns) as usize;
        c_over = max(c_over, self.min_chunk_size);

        let waves = match variability_pct {
            v if v < 25 => 2,
            v if v < 75 => 4,
            _ => 8,
        };

        let c_bal = match size_hint.1 {
            Some(remaining) if remaining > 0 => {
                max(1, remaining / (self.max_num_threads.max(1) * waves))
            }
            _ => Self::fallback_balance_bound(variability_pct),
        };

        min(min(c_bal, c_over), 1024).max(self.min_chunk_size)
    }

    pub(super) fn selected_chunk_size(&self, size_hint: (usize, Option<usize>)) -> usize {
        let remaining = size_hint.1.unwrap_or(size_hint.0).max(1);
        let chosen = match self.chosen_chunk_size.load(Ordering::Relaxed) {
            0 => {
                let chunk_size = self.choose_fixed_chunk_size(size_hint);
                self.chosen_chunk_size.store(chunk_size, Ordering::Relaxed);
                chunk_size
            }
            chunk_size => chunk_size,
        };

        min(chosen, remaining)
    }
}
