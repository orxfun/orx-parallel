use crate::runner::runner_variants::run_b::ewma::EwmaParams;

/// Minimum wall-clock exploration time before chunk-size is fixed.
pub const EXPLORATION_MIN_MS: u128 = 5;

/// Exploration stops early once this percentage of items has been sampled.
pub const EXPLORATION_TARGET_PCT: usize = 2;

/// Exploration is unconditionally capped at this percentage of items.
pub const EXPLORATION_CAP_PCT: usize = 10;

/// Estimated fixed overhead per chunk dispatch (ns), used for amortization.
pub const OVERHEAD_NS_PER_CHUNK: u64 = 2_000;

/// Chunk size is chosen so that per-item work amortizes dispatch overhead by this factor.
pub const OVERHEAD_AMORTIZATION_FACTOR: u64 = 20;

/// Target minimum per-item work (ns) for a chunk to amortize dispatch overhead.
/// Equal to `OVERHEAD_AMORTIZATION_FACTOR * OVERHEAD_NS_PER_CHUNK`.
pub const AMORTIZED_OVERHEAD_NS: u64 = OVERHEAD_AMORTIZATION_FACTOR * OVERHEAD_NS_PER_CHUNK;

/// Per-item work below this threshold (ns) is considered "tiny"; exploration exits early.
pub const TINY_WORK_THRESHOLD_NS: u64 = 500;

/// Minimum number of sampled items before the tiny-work early exit is applied.
pub const TINY_WORK_MIN_SAMPLES: usize = 64;

/// Number of consecutive stable EWMA updates required to declare convergence.
pub const CONVERGENCE_THRESHOLD: usize = 5;

/// Minimum number of sampled items before convergence-based stopping is considered.
pub const CONVERGENCE_MIN_SAMPLES: usize = 96;

/// Base minimum exploration threshold. Actual threshold scales with thread count: `max(128, 8 * num_threads)`.
pub const EXPLORATION_MIN_SAMPLES_BASE: usize = 128;

/// Scaling factor for exploration threshold relative to thread count.
pub const EXPLORATION_SAMPLES_PER_THREAD: usize = 8;

/// Divisor for the convergence stability check: the EWMA average must change by less than
/// `1 / CONVERGENCE_STABILITY_DIVISOR` (i.e. 2%) between updates to count as stable.
pub const CONVERGENCE_STABILITY_DIVISOR: u64 = 50;

/// EWMA smoothing parameters for the per-item time average (α = 1/8); slow to react, filters noise.
pub const EWMA_PARAMS_AVG: EwmaParams = EwmaParams {
    numerator: 7,
    denominator: 8,
};

/// EWMA smoothing parameters for the per-item deviation (α = 1/4); faster to react, tracks variability spikes.
pub const EWMA_PARAMS_DEV: EwmaParams = EwmaParams {
    numerator: 3,
    denominator: 4,
};

/// If per-item work exceeds this multiple of `OVERHEAD_NS_PER_CHUNK`, dispatch overhead is
/// negligible and `min_chunk_size` is used directly for best load balance.
pub const HEAVY_WORK_OVERHEAD_FACTOR: u64 = 200;

/// If workload variability (deviation as % of average) exceeds this threshold, the work is
/// too heterogeneous for amortization-based sizing; `min_chunk_size` is used to limit imbalance.
pub const HIGH_VARIABILITY_PCT_THRESHOLD: u64 = 150;

/// Per-item time threshold (ns) above which dispatch overhead is negligible.
/// Equal to `HEAVY_WORK_OVERHEAD_FACTOR * OVERHEAD_NS_PER_CHUNK`.
pub const HEAVY_WORK_NS_THRESHOLD: u64 = HEAVY_WORK_OVERHEAD_FACTOR * OVERHEAD_NS_PER_CHUNK;

/// Hard upper bound on the computed chunk size; prevents excessively large chunks
/// that would hurt load balance even when overhead amortization suggests a larger value.
pub const MAX_CHUNK_SIZE: usize = 1024;

/// Returns a conservative chunk-size balance target based on workload variability.
/// Used as fallback when the total item count is unknown. Higher variability triggers smaller
/// chunks to improve load balancing and reduce tail latency.
pub fn fallback_balance_bound(variability_pct: u64) -> usize {
    match variability_pct {
        v if v < 25 => 128,
        v if v < 75 => 64,
        v if v < 150 => 16,
        _ => 4,
    }
}

/// Returns the number of scheduling waves (rounds of chunks per thread) based on workload variability.
/// More waves means smaller chunks and finer scheduling granularity, reducing thread idle time
/// when per-item durations are unpredictable.
pub fn balance_waves(variability_pct: u64) -> usize {
    match variability_pct {
        v if v < 25 => 2,
        v if v < 75 => 4,
        _ => 8,
    }
}
