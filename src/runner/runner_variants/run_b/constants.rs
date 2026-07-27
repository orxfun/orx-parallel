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
