#[cfg(feature = "std")]
use core::cmp::{max, min};
#[cfg(feature = "std")]
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
#[cfg(feature = "std")]
use std::time::Instant;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(feature = "std")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Explore,
    Fixed,
}

#[cfg(feature = "std")]
#[derive(Clone, Debug)]
/// Diagnostics data collected by RunnerB during the exploration phase.
/// Accessible via [`take_last_runner_b_diagnostics`](crate::take_last_runner_b_diagnostics)
/// after a parallel computation completes.
pub struct DiagnosticData {
    /// Samples collected during exploration: (chunk_size, elapsed_ns)
    pub samples: Vec<(u32, u64)>,
    /// When exploration phase started
    pub exploration_phase_started_at: Instant,
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
    pub collect_diagnostics: bool,
    #[cfg(feature = "std")]
    pub(crate) diagnostics: std::sync::Mutex<Option<DiagnosticData>>,
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

        // Record diagnostic sample during exploration phase (early exit if disabled)
        self.record_sample(chunk_state.requested_chunk_size as u32, elapsed_ns);
    }

    #[cfg(not(feature = "std"))]
    #[allow(dead_code)]
    pub(crate) fn record_chunk(&self, _: ChunkState) {}

    #[cfg(feature = "std")]
    pub(crate) fn record_sample(&self, chunk_size: u32, elapsed_ns: u64) {
        // Early exit if diagnostics are disabled (negligible overhead via branch prediction)
        if !self.collect_diagnostics {
            return;
        }

        // Only record samples during exploration phase
        if self.mode() != Mode::Explore {
            return;
        }

        // Record the sample; cap at 512 samples to keep overhead bounded
        const MAX_DIAGNOSTIC_SAMPLES: usize = 512;
        if let Ok(mut diag_guard) = self.diagnostics.try_lock() {
            if let Some(diag) = diag_guard.as_mut() {
                if diag.samples.len() < MAX_DIAGNOSTIC_SAMPLES {
                    diag.samples.push((chunk_size, elapsed_ns));
                }
            }
        }
        // If lock fails (contention), silently skip this sample to avoid blocking
    }

    #[cfg(not(feature = "std"))]
    #[allow(dead_code)]
    pub(crate) fn record_sample(&self, _: u32, _: u64) {}

    #[cfg(feature = "std")]
    /// Enable diagnostics collection for exploration phase.
    /// Must be called before any work is scheduled.
    pub fn enable_diagnostics(&mut self) {
        self.collect_diagnostics = true;
        // Initialize DiagnosticData if not already present
        if let Ok(mut diag_guard) = self.diagnostics.try_lock() {
            if diag_guard.is_none() {
                *diag_guard = Some(DiagnosticData {
                    samples: Vec::new(),
                    exploration_phase_started_at: self.explore_started_at,
                });
            }
        }
    }

    #[cfg(feature = "std")]
    /// Retrieve collected diagnostics, consuming them from the State.
    pub fn take_diagnostics(&self) -> Option<DiagnosticData> {
        if let Ok(mut diag_guard) = self.diagnostics.lock() {
            diag_guard.take()
        } else {
            None
        }
    }

    #[cfg(feature = "std")]
    pub(crate) fn should_stop_exploration(&self) -> bool {
        let explored = self.explored_tasks.load(Ordering::Relaxed);
        let min_samples = max(128, 8 * self.max_num_threads.max(1));
        let elapsed_ms = self.explore_started_at.elapsed().as_millis();

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
