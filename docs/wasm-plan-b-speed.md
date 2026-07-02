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
