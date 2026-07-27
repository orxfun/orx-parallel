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

/// Per-item work below this threshold (ns) is considered "tiny"; exploration exits early.
pub const TINY_WORK_THRESHOLD_NS: u64 = 500;

/// Minimum number of sampled items before the tiny-work early exit is applied.
pub const TINY_WORK_MIN_SAMPLES: usize = 64;

/// Number of consecutive stable EWMA updates required to declare convergence.
pub const CONVERGENCE_THRESHOLD: usize = 5;

/// Minimum number of sampled items before convergence-based stopping is considered.
pub const CONVERGENCE_MIN_SAMPLES: usize = 96;

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
