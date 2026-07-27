#[cfg(feature = "std")]
use core::cmp::{max, min};
#[cfg(feature = "std")]
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
#[cfg(feature = "std")]
use std::time::Instant;

#[cfg(feature = "std")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Explore,
    Fixed,
}

pub struct State {
    pub max_num_threads: usize,
    pub min_chunk_size: usize,
    pub fixed_chunk_size: Option<usize>,
    #[cfg(feature = "std")]
    pub initial_len: Option<usize>,
    #[cfg(feature = "std")]
    pub(crate) explore_started_at: Instant,
    #[cfg(feature = "std")]
    pub(crate) mode: AtomicUsize,
    #[cfg(feature = "std")]
    pub(crate) chosen_chunk_size: AtomicUsize,
    #[cfg(feature = "std")]
    pub(crate) explored_tasks: AtomicUsize,
    #[cfg(feature = "std")]
    pub(crate) avg_ns_per_item: AtomicU64,
    #[cfg(feature = "std")]
    pub(crate) avg_abs_deviation_ns_per_item: AtomicU64,
    #[cfg(feature = "std")]
    pub(crate) prev_avg_ns_per_item: AtomicU64,
    #[cfg(feature = "std")]
    pub(crate) converged_samples: AtomicUsize,
}

pub struct ChunkState {
    #[cfg(feature = "std")]
    pub requested_chunk_size: usize,
    #[cfg(feature = "std")]
    pub started_at: Instant,
}

impl ChunkState {
    #[cfg(feature = "std")]
    pub(crate) fn new(chunk_size: usize) -> Self {
        Self {
            requested_chunk_size: chunk_size,
            started_at: Instant::now(),
        }
    }

    #[cfg(not(feature = "std"))]
    pub(crate) fn new(_: usize) -> Self {
        Self {}
    }
}

#[cfg(feature = "std")]
const EXPLORATION_MIN_MS: u128 = 5;
#[cfg(feature = "std")]
const EXPLORATION_TARGET_PCT: usize = 2;
#[cfg(feature = "std")]
const EXPLORATION_CAP_PCT: usize = 10;
#[cfg(feature = "std")]
const OVERHEAD_NS_PER_CHUNK: u64 = 2_000;
#[cfg(feature = "std")]
const OVERHEAD_AMORTIZATION_FACTOR: u64 = 20;

impl State {
    #[cfg(feature = "std")]
    fn ewma(previous: u64, sample: u64, numerator: u64, denominator: u64) -> u64 {
        match previous {
            0 => sample,
            value => (value.saturating_mul(numerator) + sample) / denominator,
        }
    }

    #[cfg(feature = "std")]
    pub(crate) fn mode(&self) -> Mode {
        match self.mode.load(Ordering::Relaxed) {
            0 => Mode::Explore,
            _ => Mode::Fixed,
        }
    }

    #[cfg(feature = "std")]
    pub(crate) fn complete_exploration(&self) {
        self.mode.store(1, Ordering::Relaxed);
    }

    #[cfg(feature = "std")]
    pub(crate) fn record_chunk(&self, chunk_state: ChunkState) {
        let elapsed_ns = chunk_state
            .started_at
            .elapsed()
            .as_nanos()
            .min(u64::MAX as u128) as u64;
        let items = chunk_state.requested_chunk_size.max(1) as u64;
        let sample_ns_per_item = elapsed_ns / items;

        let previous_avg = self.avg_ns_per_item.load(Ordering::Relaxed);
        let updated_avg = Self::ewma(previous_avg, sample_ns_per_item, 7, 8);
        self.avg_ns_per_item.store(updated_avg, Ordering::Relaxed);

        let deviation = updated_avg.abs_diff(sample_ns_per_item);
        let previous_dev = self.avg_abs_deviation_ns_per_item.load(Ordering::Relaxed);
        let updated_dev = Self::ewma(previous_dev, deviation, 3, 4);
        self.avg_abs_deviation_ns_per_item
            .store(updated_dev, Ordering::Relaxed);

        self.explored_tasks
            .fetch_add(chunk_state.requested_chunk_size.max(1), Ordering::Relaxed);

        // Track convergence: if average is stable (changed < 2%), increment counter
        let prev_avg = self.prev_avg_ns_per_item.load(Ordering::Relaxed);
        if prev_avg > 0 {
            let change = updated_avg.abs_diff(prev_avg);
            let threshold = prev_avg / 50; // 2% change threshold
            if change < threshold {
                self.converged_samples.fetch_add(1, Ordering::Relaxed);
            } else {
                // Reset convergence counter if change exceeds threshold
                self.converged_samples.store(0, Ordering::Relaxed);
            }
        }
        self.prev_avg_ns_per_item
            .store(updated_avg, Ordering::Relaxed);
    }

    #[cfg(not(feature = "std"))]
    #[allow(dead_code)]
    pub(crate) fn record_chunk(&self, _: ChunkState) {}

    #[cfg(feature = "std")]
    pub(crate) fn should_stop_exploration(&self) -> bool {
        let explored = self.explored_tasks.load(Ordering::Relaxed);
        let avg_ns = self.avg_ns_per_item.load(Ordering::Relaxed);
        let min_samples = max(128, 8 * self.max_num_threads.max(1));
        let elapsed_ms = self.explore_started_at.elapsed().as_millis();

        // Early exit for tiny work: if per-item work is extremely small,
        // exploration overhead dominates. Stop after minimal sampling.
        const TINY_WORK_THRESHOLD_NS: u64 = 500;
        const TINY_WORK_MIN_SAMPLES: usize = 64;
        if avg_ns > 0 && avg_ns < TINY_WORK_THRESHOLD_NS && explored >= TINY_WORK_MIN_SAMPLES {
            return true;
        }

        // Convergence-based stopping: if average has been stable for 5 samples,
        // and we've explored enough, stop exploring
        let converged = self.converged_samples.load(Ordering::Relaxed);
        const CONVERGENCE_THRESHOLD: usize = 5;
        const CONVERGENCE_MIN_SAMPLES: usize = 96;
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

    #[cfg(not(feature = "std"))]
    #[allow(dead_code)]
    pub(crate) fn should_stop_exploration(&self) -> bool {
        false
    }

    #[cfg(feature = "std")]
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

    #[cfg(feature = "std")]
    fn fallback_balance_bound(variability_pct: u64) -> usize {
        if variability_pct < 25 {
            128
        } else if variability_pct < 75 {
            64
        } else if variability_pct < 150 {
            16
        } else {
            4
        }
    }

    #[cfg(feature = "std")]
    pub(crate) fn choose_fixed_chunk_size(&self, size_hint: (usize, Option<usize>)) -> usize {
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

        let waves = if variability_pct < 25 {
            2
        } else if variability_pct < 75 {
            4
        } else {
            8
        };

        let c_bal = match size_hint.1 {
            Some(remaining) if remaining > 0 => {
                max(1, remaining / (self.max_num_threads.max(1) * waves))
            }
            _ => Self::fallback_balance_bound(variability_pct),
        };

        min(min(c_bal, c_over), 1024).max(self.min_chunk_size)
    }

    #[cfg(feature = "std")]
    pub(crate) fn selected_chunk_size(&self, size_hint: (usize, Option<usize>)) -> usize {
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
