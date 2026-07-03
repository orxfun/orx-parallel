# wasm-web-threads2 Speed Plan (Plan B - Performance Milestone)

## Context

`wasm_web2.rs` is now functionally working and achieves true browser parallelism (`spawned_workers > 0`) in the demo.

Current observed gap (same settings, 16 threads):

- `wasm_web` (`wasm_demo_tsp`): ~448 ms
- `wasm_web2` (`wasm_demo_tsp2`): ~704 ms

New milestone: close this performance gap while preserving correctness and stability.

---

## Milestone Goal

Primary goal:

- Improve `wasm_web2` throughput and reduce elapsed time to be competitive with `wasm_web` for browser workloads.

Target thresholds:

1. Phase target: `wasm_web2` within 25% of `wasm_web` median runtime.
2. Stretch target: `wasm_web2` within 10-15% of `wasm_web` median runtime.

Non-negotiables:

- Keep `spawned_workers > 0` in supported browser configurations.
- Keep existing correctness behavior and panic/error safety expectations.
- Avoid regressions in initialization reliability and UI responsiveness.

---

## Measurement Strategy (Low Noise, Browser-Real)

We should not rely only on manual UI clicks to evaluate speed.

### Why not CLI-only?

CLI benchmarks are useful for fast iteration, but they do not model browser worker/runtime overhead exactly.

### Recommended two-layer measurement

1. Diagnostic benchmark (CLI/native):
   - Purpose: isolate compute and scheduler overhead quickly.
   - Use: inner-loop optimization and profiling.

2. Browser-real benchmark (headless, no UI interactions):
   - Purpose: final truth for wasm web threads performance.
   - Use: milestone acceptance and regression checks.

### Benchmark protocol

For each candidate change:

1. Run `N >= 20` trials per variant.
2. Compare medians and p95 (not a single run).
3. Keep fixed inputs: seed, city count, iterations, thread count, chunking config.
4. Collect both wasm-reported elapsed and wall-clock elapsed.

---

## Performance Hypotheses to Validate

Likely sources of overhead in `wasm_web2` vs `wasm_web`:

1. Per-task queue lock contention (`Mutex<VecDeque<Task>>`).
2. High wake/notify frequency and scheduling overhead.
3. Per-task panic boundary overhead (`catch_unwind` in hot path).
4. Main-thread cooperative completion loop overhead.
5. Work granularity too fine (too many small scheduled tasks).

---

## PR Plan

## PR-1: Benchmark Harness and Perf Telemetry

Scope:

- Add reproducible benchmark harness (headless browser path + optional CLI diagnostic).
- Add minimal runtime counters in `wasm_web2` to expose scheduling behavior.

Deliverables:

1. Runtime counters API (debug/perf mode), e.g.:
   - tasks enqueued
   - tasks run by worker loop
   - tasks run by main thread
   - notify count
   - queue depth high-water mark
2. Demo-side benchmark script/entry that runs fixed workloads without manual UI.
3. Baseline report doc section with current numbers.

Acceptance:

- Numbers are reproducible across repeated runs.
- We can attribute where time is spent before changing algorithms.

---

## PR-2: Task Granularity and Scheduling Pressure Reduction

Scope:

- Reduce number of scheduled tasks for same logical work.
- Tune chunking policy for wasm-web-threads2 path.

Deliverables:

1. Coarser task/chunk policy (without changing output correctness).
2. Updated benchmark comparisons and telemetry deltas.

Acceptance:

- Significant drop in scheduled task count.
- Measurable median runtime improvement in browser benchmark.

---

## PR-3: Queue and Wakeup Optimization

Scope:

- Reduce synchronization bottlenecks in task handoff.

Candidate options:

1. Keep current queue but batch push/pop and reduce notify frequency.
2. Introduce a better concurrent queue strategy (if complexity justified).
3. Reduce lock hold durations and avoid unnecessary state transitions.

Acceptance:

- Lower lock-contention-related telemetry.
- Better scaling between 8 and 16 workers.

---

## PR-4: Hot-Path Overhead Cleanup

Scope:

- Minimize per-task overhead in critical paths.

Candidate options:

1. Revisit `catch_unwind` placement (scope-level vs per-task under safe constraints).
2. Avoid repeated runtime/state lookups in tight loops.
3. Reduce allocation churn for task wrappers where possible.

Acceptance:

- Improved iterations/s with no behavior regressions.

---

## PR-5: Browser Loop and Completion Strategy Tuning

Scope:

- Tune completion waiting and cooperative execution strategy to reduce waste.

Candidate options:

1. Smarter main-thread helping strategy (less spin overhead).
2. Better balance between worker execution and main-thread assist.
3. Guard against pathological wake/spin cycles under small workloads.

Acceptance:

- Lower p95 and more stable run-to-run variance.

---

## PR-6: Final Comparison, Docs, and Regression Gates

Scope:

- Finalize benchmark methodology and expected thresholds.
- Document tuning knobs and interpretation.

Deliverables:

1. Final comparison table (`wasm_web` vs `wasm_web2`).
2. Guidance on recommended thread/chunk settings.
3. Optional CI perf-smoke command (non-blocking threshold initially).

Acceptance:

- Clear evidence of milestone attainment (or explicit quantified gap if not yet reached).
- Team has repeatable process to prevent performance regressions.

---

## Risks and Mitigations

1. Risk: optimize for synthetic benchmark only.
   - Mitigation: always validate in browser-real harness.

2. Risk: correctness regressions due to aggressive scheduler changes.
   - Mitigation: retain/expand smoke tests and deterministic run checks.

3. Risk: platform variance hides gains.
   - Mitigation: compare medians and p95 over repeated runs; record environment metadata.

---

## Definition of Done

This milestone is complete when:

1. `wasm_web2` remains functionally stable (`spawned_workers > 0`, no startup/run regressions).
2. Browser benchmark shows sustained improvement over baseline.
3. Performance gap to `wasm_web` is reduced to target threshold (phase or stretch).
4. Benchmarking process is documented and repeatable for future work.

---

## PR-1 Implementation Status

Implemented in this milestone kickoff:

1. wasm-web-threads2 runtime telemetry counters are now available:
   - tasks enqueued
   - tasks run by workers
   - tasks run by main thread
   - notify count
   - queue depth high-water mark
2. Demo crate exports telemetry APIs and a fixed-workload benchmark report API.
3. A dedicated no-UI benchmark entry page was added (`benchmark.html`) with a script that runs fixed sequential and parallel trials and prints a JSON report.

How to run benchmark harness:

1. `cd examples/wasm_demo_tsp2/web`
2. `npm run bench:full`
3. Read JSON report from the benchmark page (and browser console).

Report includes:

1. runtime worker info (`configured_threads`, `spawned_workers`)
2. sequential trial summary (wall/wasm medians and p95)
3. parallel summary with wasm-web2 telemetry snapshot

### PR-1 Baseline Sample (2026-07-02)

```json
{
   "config": {
      "trials": 20,
      "iterations": 10000,
      "threads": 16,
      "numCities": 50,
      "seed": "42"
   },
   "runtime": {
      "configured_threads": 16,
      "spawned_workers": 16
   },
   "sequential": {
      "wall": {
         "median_ms": 13299.269999995828,
         "p95_ms": 16704.125,
         "mean_ms": 13227.018750000745,
         "min_ms": 8640.130000010133,
         "max_ms": 17082.819999992847
      },
      "wasm": {
         "median_ms": 13299,
         "p95_ms": 16704,
         "mean_ms": 13226.95,
         "min_ms": 8640,
         "max_ms": 17083
      }
   },
   "parallel": {
      "trials": 20,
      "iterations_per_trial": 10000,
      "threads": 16,
      "num_cities": 50,
      "median_ms": 3225,
      "p95_ms": 3710,
      "mean_ms": 3298.35,
      "min_ms": 2621,
      "max_ms": 5195,
      "perf": {
         "tasks_enqueued": 160,
         "tasks_run_by_workers": 160,
         "tasks_run_by_main": 0,
         "notify_count": 160,
         "queue_depth_high_water": 1
      }
   }
}
```

Interpretation:

1. True worker parallelism is active (`spawned_workers=16`, `tasks_run_by_main=0`).
2. Queue occupancy is very low (`queue_depth_high_water=1`) with relatively few tasks per trial.
3. PR-2 should focus on improving worker occupancy by tuning chunk granularity, then re-benchmark.

### PR-2 Chunk Size Sweep Sample (2026-07-03)

```json
{
   "config": {
      "trials": 20,
      "iterations": 10000,
      "threads": 16,
      "chunkSizes": [
         0,
         1,
         2,
         4,
         8,
         16,
         32,
         64,
         128,
         256
      ],
      "numCities": 50,
      "seed": "42"
   },
   "runtime": {
      "configured_threads": 16,
      "spawned_workers": 16
   },
   "sequential": {
      "wall": {
         "median_ms": 7044.274999991059,
         "p95_ms": 21749.969999998808,
         "mean_ms": 8399.522749999165,
         "min_ms": 5774.655000001192,
         "max_ms": 22387.62000000477
      },
      "wasm": {
         "median_ms": 7044,
         "p95_ms": 21749,
         "mean_ms": 8399.45,
         "min_ms": 5774,
         "max_ms": 22387
      }
   },
   "parallel": [
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 0,
         "num_cities": 50,
         "median_ms": 11066,
         "p95_ms": 12855,
         "mean_ms": 11009.9,
         "min_ms": 2976,
         "max_ms": 13212,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 159,
            "tasks_run_by_main": 1,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 1,
         "num_cities": 50,
         "median_ms": 3312,
         "p95_ms": 3960,
         "mean_ms": 3453.7,
         "min_ms": 2908,
         "max_ms": 4123,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 159,
            "tasks_run_by_main": 1,
            "notify_count": 160,
            "queue_depth_high_water": 2
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 2,
         "num_cities": 50,
         "median_ms": 3274,
         "p95_ms": 4043,
         "mean_ms": 3468.45,
         "min_ms": 3000,
         "max_ms": 4104,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 160,
            "tasks_run_by_main": 0,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 4,
         "num_cities": 50,
         "median_ms": 3294,
         "p95_ms": 3685,
         "mean_ms": 3432.75,
         "min_ms": 2924,
         "max_ms": 4045,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 160,
            "tasks_run_by_main": 0,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 8,
         "num_cities": 50,
         "median_ms": 3600,
         "p95_ms": 3671,
         "mean_ms": 3467.9,
         "min_ms": 2954,
         "max_ms": 4070,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 160,
            "tasks_run_by_main": 0,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 16,
         "num_cities": 50,
         "median_ms": 3301,
         "p95_ms": 3752,
         "mean_ms": 3399.2,
         "min_ms": 2960,
         "max_ms": 4159,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 159,
            "tasks_run_by_main": 1,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 32,
         "num_cities": 50,
         "median_ms": 3517,
         "p95_ms": 4305,
         "mean_ms": 3617.65,
         "min_ms": 2992,
         "max_ms": 4357,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 159,
            "tasks_run_by_main": 1,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 64,
         "num_cities": 50,
         "median_ms": 3428,
         "p95_ms": 4021,
         "mean_ms": 3517.05,
         "min_ms": 3010,
         "max_ms": 4385,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 160,
            "tasks_run_by_main": 0,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 128,
         "num_cities": 50,
         "median_ms": 3840,
         "p95_ms": 5097,
         "mean_ms": 4029.6,
         "min_ms": 3070,
         "max_ms": 5104,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 160,
            "tasks_run_by_main": 0,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 256,
         "num_cities": 50,
         "median_ms": 4437,
         "p95_ms": 4857,
         "mean_ms": 4424.1,
         "min_ms": 3029,
         "max_ms": 4933,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 160,
            "tasks_run_by_main": 0,
            "notify_count": 160,
            "queue_depth_high_water": 1
         }
      }
   ]
}
```

Interpretation:

1. Auto chunking (`chunk_size=0`) is a major regression in this workload (median ~11066 ms).
2. Fixed chunk sizes between 1 and 16 are all strong and close, with best median at `chunk_size=2` (~3274 ms).
3. Larger chunks (>= 32) degrade performance progressively, confirming PR-2 should focus on small-chunk behavior and safer auto policy for wasm-web-threads2.

### PR-2 Chunk Size Sweep Re-Run (2026-07-03, after setting demos to chunk size 1)

```json
{
   "config": {
      "trials": 20,
      "iterations": 10000,
      "threads": 16,
      "chunkSizes": [
         0,
         1,
         2,
         4,
         8,
         16,
         32,
         64,
         128,
         256
      ],
      "numCities": 50,
      "seed": "42"
   },
   "runtime": {
      "configured_threads": 16,
      "spawned_workers": 16
   },
   "sequential": {
      "wall": {
         "median_ms": 10005.964999973774,
         "p95_ms": 21864.52000001073,
         "mean_ms": 13253.319499996305,
         "min_ms": 9047.125,
         "max_ms": 21896.559999972582
      },
      "wasm": {
         "median_ms": 10006,
         "p95_ms": 21864,
         "mean_ms": 13253.2,
         "min_ms": 9047,
         "max_ms": 21895
      }
   },
   "parallel": [
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 0,
         "num_cities": 50,
         "median_ms": 16287,
         "p95_ms": 19184,
         "mean_ms": 16559.8,
         "min_ms": 14600,
         "max_ms": 19572,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 6
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 1,
         "num_cities": 50,
         "median_ms": 7501,
         "p95_ms": 8802,
         "mean_ms": 7585,
         "min_ms": 6467,
         "max_ms": 10043,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 6
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 2,
         "num_cities": 50,
         "median_ms": 6765,
         "p95_ms": 7816,
         "mean_ms": 6867.3,
         "min_ms": 6173,
         "max_ms": 7943,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 7
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 4,
         "num_cities": 50,
         "median_ms": 7020,
         "p95_ms": 8640,
         "mean_ms": 7309.75,
         "min_ms": 6288,
         "max_ms": 9673,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 5
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 8,
         "num_cities": 50,
         "median_ms": 6443,
         "p95_ms": 7174,
         "mean_ms": 6660.1,
         "min_ms": 6231,
         "max_ms": 7960,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 7
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 16,
         "num_cities": 50,
         "median_ms": 6797,
         "p95_ms": 7621,
         "mean_ms": 6809.7,
         "min_ms": 6271,
         "max_ms": 7908,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 6
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 32,
         "num_cities": 50,
         "median_ms": 6988,
         "p95_ms": 7795,
         "mean_ms": 7143.05,
         "min_ms": 6294,
         "max_ms": 10695,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 141,
            "tasks_run_by_main": 19,
            "notify_count": 160,
            "queue_depth_high_water": 6
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 64,
         "num_cities": 50,
         "median_ms": 7265,
         "p95_ms": 8094,
         "mean_ms": 7256.55,
         "min_ms": 6537,
         "max_ms": 8173,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 5
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 128,
         "num_cities": 50,
         "median_ms": 7536,
         "p95_ms": 8843,
         "mean_ms": 7784.8,
         "min_ms": 6554,
         "max_ms": 9923,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 6
         }
      },
      {
         "trials": 20,
         "iterations_per_trial": 10000,
         "threads": 16,
         "chunk_size": 256,
         "num_cities": 50,
         "median_ms": 8417,
         "p95_ms": 9054,
         "mean_ms": 8109.9,
         "min_ms": 6794,
         "max_ms": 9314,
         "perf": {
            "tasks_enqueued": 160,
            "tasks_run_by_workers": 140,
            "tasks_run_by_main": 20,
            "notify_count": 160,
            "queue_depth_high_water": 7
         }
      }
   ]
}
```

Interpretation (deeper):

1. Chunk size still matters (`chunk_size=0` is worst), but even tuned chunk sizes remain far slower than earlier best-case runs, so chunking alone is not the main remaining gap.
2. Telemetry shape changed materially: `tasks_run_by_main=20` and `tasks_run_by_workers=140` for almost all chunk sizes. That means the main thread is consistently executing one task per trial, indicating persistent worker-side underutilization or scheduler handoff overhead.
3. `queue_depth_high_water` increased from ~1 in earlier runs to ~5-7, pointing to more queue accumulation and higher contention/backpressure.

Likely underlying reasons for `wasm_demo_tsp2` vs `wasm_demo_tsp` gap (after controlling chunk size):

1. Scheduler architecture overhead: `wasm_web2` currently uses a central queue and explicit wake/notify protocol, while `wasm_web` rides Rayon's mature work-stealing scheduler. Even with same chunk size, central handoff can cost more.
2. Main-thread assist cost: the consistent `tasks_run_by_main=20` pattern suggests completion is relying on main-thread participation every trial, which adds coordination overhead and can steal time from orchestration.
3. Synchronization strategy overhead: `wasm_web2` has multiple shared counters/state transitions and lock/spin paths in hot loops. This can dominate when each task body is expensive and long-lived.
4. Task-topology ceiling: with `tasks_enqueued=160` for 20 trials, effective task fan-out is only 8 tasks/trial for 16 workers. This limits worker saturation independently of iterator chunk granularity.
5. Worker readiness/wakeup dynamics: higher queue depth with nonzero main-thread execution can indicate workers are not draining as quickly as enqueuing pace, implying wakeup latency or scheduling imbalance.

Suggested next diagnostic step (separate PR):

1. Add per-worker run counters and per-trial timeline marks (enqueue-start/end, first worker start, completion) to isolate whether the gap is dominated by wakeup latency, lock contention, or low fan-out.

### Diagnostic Checklist (Implementation-Ready)

Goal:

1. Identify which mechanism dominates the remaining gap after chunk-size control: wakeup latency, synchronization overhead, low fan-out, or load imbalance.

Scope:

1. Keep this as instrumentation only (no scheduling logic changes in same PR).
2. Guard with a debug/perf cfg path so release hot path remains minimal.

#### A) Add Counters in `wasm_web2` Runtime

Add or extend these counters (atomics):

1. Worker activity:
   - `worker_runs_total`
   - `worker_runs_by_id[worker_id]`
   - `worker_steals_total` (or queue pop attempts if no steal concept)
2. Main-thread assist:
   - `main_assist_runs_total`
   - `main_assist_time_ns_total`
3. Queue/scheduler pressure:
   - `queue_push_total`
   - `queue_pop_total`
   - `queue_empty_poll_total`
   - `notify_total` (already exists)
   - `wake_wait_total`
   - `wake_spurious_total` (woke but no task)
4. Synchronization cost proxies:
   - `state_try_lock_fail_total`
   - `state_try_lock_spin_iters_total`
   - `completion_notify_total`
5. Trial-level shape:
   - `tasks_per_trial_total`
   - `trials_with_main_assist`

#### B) Add Per-Trial Timeline Marks

Capture monotonic timestamps (ns/ms) for each trial:

1. `t_trial_start`
2. `t_enqueue_done`
3. `t_first_worker_task_start`
4. `t_last_worker_task_end`
5. `t_completion_observed`

From these, derive:

1. `enqueue_phase = t_enqueue_done - t_trial_start`
2. `worker_start_latency = t_first_worker_task_start - t_enqueue_done`
3. `worker_active_span = t_last_worker_task_end - t_first_worker_task_start`
4. `completion_tail = t_completion_observed - t_last_worker_task_end`
5. `trial_total = t_completion_observed - t_trial_start`

#### C) Add Derived Metrics to Benchmark Report

Expose in benchmark JSON:

1. `main_assist_share = main_assist_runs_total / tasks_enqueued`
2. `worker_utilization_skew`:
   - max/min or stddev of `worker_runs_by_id`
3. `queue_pressure_index`:
   - `queue_empty_poll_total / queue_pop_total`
4. `lock_contention_index`:
   - `state_try_lock_fail_total / queue_pop_total`
5. Timeline medians/p95 for:
   - `worker_start_latency`
   - `completion_tail`

#### D) Minimal Surfacing API

1. Keep existing perf snapshot API and add:
   - `wasm_web2_perf_snapshot_extended()` for new counters.
   - `wasm_web2_perf_trial_samples()` returning compact per-trial durations.
2. In demo benchmark report, include both basic and extended sections.

#### E) Run Matrix (to isolate causes)

Run at fixed seed/cities/iterations/threads:

1. Chunk-size sweep: `{1, 2, 4, 8, 16}`.
2. Thread sweep at fixed chunk (best from above): `{4, 8, 16}`.
3. Repeat each with `N >= 20` trials.

Compare against `wasm_demo_tsp` baseline using same workload.

#### F) Decision Rules (what result means)

1. If `worker_start_latency` is high and `wake_spurious_total` high:
   - wake/notify path is primary bottleneck.
2. If `lock_contention_index` high and `state_try_lock_spin_iters_total` grows with threads:
   - synchronization path is primary bottleneck.
3. If `worker_utilization_skew` is high with low queue pressure:
   - load balancing/fan-out is primary bottleneck.
4. If `completion_tail` is high with frequent main assist:
   - completion coordination/main-thread help path is primary bottleneck.

#### G) Acceptance for Diagnostic PR

1. We can classify one dominant bottleneck with evidence from counters + timelines.
2. We can explain why tuned chunk sizes still leave a gap vs `wasm_demo_tsp`.
3. We can define a targeted follow-up optimization PR with measurable acceptance criteria.
