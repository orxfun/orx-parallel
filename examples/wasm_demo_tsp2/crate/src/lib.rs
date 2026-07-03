use serde::Serialize;
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
mod computation;
mod locations;

#[cfg_attr(
    not(all(target_arch = "wasm32", target_feature = "atomics")),
    allow(dead_code)
)]
#[derive(Debug, Serialize)]
struct RunResult {
    best_tour: Vec<usize>,
    best_distance: f64,
    iterations: usize,
    elapsed_ms: f64,
}

#[cfg_attr(
    not(all(target_arch = "wasm32", target_feature = "atomics")),
    allow(dead_code)
)]
#[derive(Debug, Serialize)]
struct RuntimeInfo {
    configured_threads: usize,
    spawned_workers: usize,
}

#[cfg_attr(
    not(all(target_arch = "wasm32", target_feature = "atomics")),
    allow(dead_code)
)]
#[derive(Debug, Serialize)]
struct PerfSnapshot {
    tasks_enqueued: usize,
    tasks_run_by_workers: usize,
    tasks_run_by_main: usize,
    notify_count: usize,
    queue_depth_high_water: usize,
}

#[cfg_attr(
    not(all(target_arch = "wasm32", target_feature = "atomics")),
    allow(dead_code)
)]
#[derive(Debug, Serialize)]
struct PerfSnapshotExtended {
    queue_pop_count: usize,
    queue_empty_poll_count: usize,
    main_assist_time_ns: usize,
    state_try_lock_fail_count: usize,
    state_try_lock_spin_iters: usize,
    completion_notify_count: usize,
    main_assist_attempt_count: usize,
    main_assist_success_count: usize,
    worker_runs_by_id: Vec<usize>,
}

#[cfg_attr(
    not(all(target_arch = "wasm32", target_feature = "atomics")),
    allow(dead_code)
)]
#[derive(Debug, Serialize)]
struct BenchmarkReport {
    trials: usize,
    iterations_per_trial: usize,
    threads: usize,
    chunk_size: usize,
    num_cities: usize,
    median_ms: f64,
    p95_ms: f64,
    mean_ms: f64,
    min_ms: f64,
    max_ms: f64,
    perf: PerfSnapshot,
    perf_extended: PerfSnapshotExtended,
    trial_samples_ms: Vec<f64>,
}

/// Initializes the wasm worker thread pool used by parallel runs.
#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    orx_parallel::init_thread_pool(num_threads as usize)
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
pub fn parallel_runtime_info() -> Result<JsValue, JsValue> {
    let (configured_threads, spawned_workers) = orx_parallel::wasm_web2_runtime_info();
    let info = RuntimeInfo {
        configured_threads,
        spawned_workers,
    };
    serde_wasm_bindgen::to_value(&info)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize runtime info: {e}")))
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
pub fn parallel_perf_reset() {
    orx_parallel::wasm_web2_perf_reset();
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
pub fn parallel_perf_snapshot() -> Result<JsValue, JsValue> {
    let (
        tasks_enqueued,
        tasks_run_by_workers,
        tasks_run_by_main,
        notify_count,
        queue_depth_high_water,
    ) = orx_parallel::wasm_web2_perf_snapshot();
    let snapshot = PerfSnapshot {
        tasks_enqueued,
        tasks_run_by_workers,
        tasks_run_by_main,
        notify_count,
        queue_depth_high_water,
    };
    serde_wasm_bindgen::to_value(&snapshot)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize perf snapshot: {e}")))
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
pub fn parallel_perf_snapshot_extended() -> Result<JsValue, JsValue> {
    let ext = orx_parallel::wasm_web2_perf_snapshot_extended();
    let snapshot = PerfSnapshotExtended {
        queue_pop_count: ext.queue_pop_count,
        queue_empty_poll_count: ext.queue_empty_poll_count,
        main_assist_time_ns: ext.main_assist_time_ns,
        state_try_lock_fail_count: ext.state_try_lock_fail_count,
        state_try_lock_spin_iters: ext.state_try_lock_spin_iters,
        completion_notify_count: ext.completion_notify_count,
        main_assist_attempt_count: ext.main_assist_attempt_count,
        main_assist_success_count: ext.main_assist_success_count,
        worker_runs_by_id: ext.worker_runs_by_id,
    };
    serde_wasm_bindgen::to_value(&snapshot)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize extended perf snapshot: {e}")))
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
pub fn run_parallel_benchmark_report(
    trials: u32,
    iterations: u32,
    seed: u64,
    threads: u32,
    chunk_size: u32,
    num_cities: u32,
) -> Result<JsValue, JsValue> {
    let trials = trials.max(1) as usize;
    let iterations = iterations.max(1) as usize;
    let threads = threads.max(1) as usize;
    let chunk_size = chunk_size as usize;
    let num_cities = locations::clamp_num_cities(num_cities);
    let locations = locations::locations(num_cities as u32);

    orx_parallel::wasm_web2_perf_reset();

    let mut samples = Vec::with_capacity(trials);
    let mut start_index = 0u64;

    for _ in 0..trials {
        let started_at = js_sys::Date::now();
        let output = computation::run_search_parallel_with_chunk_size(
            iterations,
            seed,
            threads,
            &locations,
            start_index,
            (chunk_size > 0).then_some(chunk_size),
        );
        let elapsed_ms = js_sys::Date::now() - started_at;
        if output.best.is_none() {
            return Err(JsValue::from_str(
                "parallel benchmark produced no best tour",
            ));
        }
        samples.push(elapsed_ms);
        start_index = start_index.wrapping_add(iterations as u64);
    }

    let mut sorted = samples.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

    let median_ms = percentile(&sorted, 50);
    let p95_ms = percentile(&sorted, 95);
    let sum_ms: f64 = samples.iter().copied().sum();
    let mean_ms = sum_ms / samples.len() as f64;
    let min_ms = sorted.first().copied().unwrap_or(0.0);
    let max_ms = sorted.last().copied().unwrap_or(0.0);

    let (
        tasks_enqueued,
        tasks_run_by_workers,
        tasks_run_by_main,
        notify_count,
        queue_depth_high_water,
    ) = orx_parallel::wasm_web2_perf_snapshot();
    let perf_extended = orx_parallel::wasm_web2_perf_snapshot_extended();

    let report = BenchmarkReport {
        trials,
        iterations_per_trial: iterations,
        threads,
        chunk_size,
        num_cities,
        median_ms,
        p95_ms,
        mean_ms,
        min_ms,
        max_ms,
        perf: PerfSnapshot {
            tasks_enqueued,
            tasks_run_by_workers,
            tasks_run_by_main,
            notify_count,
            queue_depth_high_water,
        },
        perf_extended: PerfSnapshotExtended {
            queue_pop_count: perf_extended.queue_pop_count,
            queue_empty_poll_count: perf_extended.queue_empty_poll_count,
            main_assist_time_ns: perf_extended.main_assist_time_ns,
            state_try_lock_fail_count: perf_extended.state_try_lock_fail_count,
            state_try_lock_spin_iters: perf_extended.state_try_lock_spin_iters,
            completion_notify_count: perf_extended.completion_notify_count,
            main_assist_attempt_count: perf_extended.main_assist_attempt_count,
            main_assist_success_count: perf_extended.main_assist_success_count,
            worker_runs_by_id: perf_extended.worker_runs_by_id,
        },
        trial_samples_ms: samples,
    };

    serde_wasm_bindgen::to_value(&report)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize benchmark report: {e}")))
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
pub fn parallel_runtime_info() -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "parallel_runtime_info is only available for wasm32 + atomics builds",
    ))
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
pub fn parallel_perf_reset() {}

#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
pub fn parallel_perf_snapshot() -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "parallel_perf_snapshot is only available for wasm32 + atomics builds",
    ))
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
pub fn parallel_perf_snapshot_extended() -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "parallel_perf_snapshot_extended is only available for wasm32 + atomics builds",
    ))
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
pub fn run_parallel_benchmark_report(
    _trials: u32,
    _iterations: u32,
    _seed: u64,
    _threads: u32,
    _chunk_size: u32,
    _num_cities: u32,
) -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "run_parallel_benchmark_report is only available for wasm32 + atomics builds",
    ))
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn percentile(sorted_samples: &[f64], p: usize) -> f64 {
    if sorted_samples.is_empty() {
        return 0.0;
    }
    let n = sorted_samples.len();
    let idx = ((n - 1) * p) / 100;
    sorted_samples[idx]
}

/// Returns an error when thread-pool initialization is unavailable on this target.
#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
pub fn init_parallel_runtime(_num_threads: u32) -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "init_parallel_runtime is only available for wasm32 + atomics builds",
    ))
}

/// Returns the city coordinates for the requested problem size.
#[wasm_bindgen]
pub fn locations(num_cities: u32) -> Result<JsValue, JsValue> {
    let locations = locations::locations(num_cities);
    serde_wasm_bindgen::to_value(&locations)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

/// Runs a parallel TSP search chunk and returns the best tour found in that chunk.
#[wasm_bindgen]
pub fn run_best_tour_par(
    iterations: u32,
    seed: u64,
    threads: u32,
    num_cities: u32,
    start_index: u64,
) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (iterations, seed, threads, num_cities, start_index);
        return Err(JsValue::from_str(
            "run_best_tour_par requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let threads = threads.max(1) as usize;
        let num_cities = locations::clamp_num_cities(num_cities);
        let locations = locations::locations(num_cities as u32);
        let started_at = js_sys::Date::now();
        let output =
            computation::run_search_parallel(iterations, seed, threads, &locations, start_index);
        let elapsed_ms = js_sys::Date::now() - started_at;
        run_output_to_js(output, elapsed_ms)
    }
}

/// Runs a sequential TSP search chunk and returns the best tour found in that chunk.
#[wasm_bindgen]
pub fn run_best_tour_seq(
    iterations: u32,
    seed: u64,
    num_cities: u32,
    start_index: u64,
) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (iterations, seed, num_cities, start_index);
        return Err(JsValue::from_str(
            "run_best_tour_seq requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        let iterations = iterations.max(1) as usize;
        let num_cities = locations::clamp_num_cities(num_cities);
        let locations = locations::locations(num_cities as u32);
        let started_at = js_sys::Date::now();
        let output = computation::run_search_sequential(iterations, seed, &locations, start_index);
        let elapsed_ms = js_sys::Date::now() - started_at;
        run_output_to_js(output, elapsed_ms)
    }
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
fn run_output_to_js(
    output: computation::SearchRunOutput,
    elapsed_ms: f64,
) -> Result<JsValue, JsValue> {
    match output.best {
        Some((best_tour, best_distance)) => {
            let result = RunResult {
                best_tour,
                best_distance,
                iterations: output.iterations,
                elapsed_ms,
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("failed to serialize result: {e}")))
        }
        None => Err(JsValue::from_str(
            "no tour could be generated (unexpected empty search)",
        )),
    }
}
