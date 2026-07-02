# Wasm Plan B: Remove rayon-core / wasm-bindgen-rayon Dependency for Web Threads

This document proposes a migration path for wasm web-threaded execution that keeps current behavior intact while adding a new runtime path that does not depend on `rayon-core` or `wasm-bindgen-rayon`.

## Goals

1. Keep existing implementation and feature:
- existing module: `src/pool/pool_impl/wasm_web.rs`
- existing feature: `wasm-web-threads`

2. Add new implementation and feature:
- new module: `src/pool/pool_impl/wasm_web2.rs`
- new feature: `wasm-web-threads2`

3. New path (`wasm-web-threads2`) must not depend on:
- `rayon-core`
- `wasm-bindgen-rayon`

4. When new feature is active on wasm, make new pool the default:
- `DefaultPool = WasmWebPool2` under `wasm-web-threads2`

5. Preserve startup model inspired by current wasm flow:
- call and await `init_thread_pool(...)` before parallel runs

6. Keep user-facing usage similar to `wasm_demo_tsp`:
- same high-level API and startup sequence
- only underlying pool/runtime implementation changes

7. Add a new demo:
- `examples/wasm_demo_tsp2`

## Current State Snapshot

- Feature wiring currently enables wasm web-thread support via `wasm-web-threads` in `Cargo.toml` and pulls in `rayon-core` and `wasm-bindgen-rayon`.
- `DefaultPool` selection currently maps wasm-threaded builds to `WasmWebPool` in `src/pool/mod.rs`.
- `WasmWebPool` in `src/pool/pool_impl/wasm_web.rs` uses:
  - `rayon_core::scope` for scoped execution
  - `wasm_bindgen_rayon::init_thread_pool(...)` for initialization
- `wasm_demo_tsp` startup sequence already follows this model:
  1. wasm module init
  2. await runtime init function
  3. run parallel computation

## Proposed Architecture

Keep the `ParThreadPool` abstraction unchanged and add a second wasm backend implementation.

- Legacy backend (unchanged):
  - feature: `wasm-web-threads`
  - module: `wasm_web.rs`
  - runtime: rayon-backed

- New backend:
  - feature: `wasm-web-threads2`
  - module: `wasm_web2.rs`
  - runtime: internal worker/thread runtime and scheduler, no rayon dependencies

This preserves external API style while allowing internal implementation replacement.

## Phase Plan

## Phase 1: Feature Graph and Conditional Exports

### 1. Cargo feature additions (`Cargo.toml`)

- Add `wasm-web-threads2` feature.
- Include only wasm interop dependencies required by the new runtime.
- Ensure `wasm-web-threads2` does not include `rayon-core` / `wasm-bindgen-rayon`.
- Keep `wasm-web-threads` unchanged for backward compatibility.

### 2. Pool module exports (`src/pool/pool_impl/mod.rs`, `src/pool/mod.rs`)

- Add gated module/export for `WasmWebPool2`.
- Add gated re-export for `init_thread_pool` under `wasm-web-threads2`.
- Introduce deterministic feature conflict behavior when both wasm features are enabled:
  - recommended: compile-time error to enforce one backend per build.

### 3. Default pool precedence (`src/pool/mod.rs`)

- Set `DefaultPool = WasmWebPool2` under:
  - `target_arch = "wasm32"`
  - `feature = "wasm-web-threads2"`
- Keep existing `DefaultPool` paths for other targets/features.

### 4. Pool factory API (`src/pool/new_pool.rs`)

- Add `Pool::wasm_web2(...) -> WasmWebPool2`.
- Keep `Pool::wasm_web(...)` intact.

## Phase 2: New Wasm Runtime Bootstrap (No Rayon)

Implement a minimal internal runtime bootstrap for web workers.

### Responsibilities

- Provide `init_thread_pool(num_threads) -> js_sys::Promise`.
- Initialize worker infrastructure and shared memory/thread metadata.
- Persist initialized state (fail-fast if not initialized before parallel use).
- Preserve atomics-required safety checks and error messages.

### Design guidance

- Follow lifecycle choices similar to `wasm-bindgen-rayon`:
  - one-time async initialization at startup
  - runtime ready gate before any scoped parallel work
- Keep implementation internal to `orx-parallel`; no rayon runtime dependency.

## Phase 3: Implement `WasmWebPool2` (`src/pool/pool_impl/wasm_web2.rs`)

Implement `ParThreadPool` for new wasm backend.

### Required behavior

- Scoped task execution semantics compatible with current iterator runners.
- Respect existing thread-count logic:
  - `NumThreads::Auto`
  - `NumThreads::Max(n)`
  - upper-bound handling via `ParThreadPool::max_num_threads_for_computation`
- Panic/fail-fast behavior when runtime is uninitialized.

### API compatibility

- Preserve the same high-level usage pattern in user code:
  - optional explicit pool via `.pool(Pool::wasm_web2(...))`
  - default pool path when `wasm-web-threads2` is active

## Phase 4: Demo Compatibility (`examples/wasm_demo_tsp2`)

Create a second demo mirroring `wasm_demo_tsp` ergonomics while using the new backend.

### Deliverables

- New crate and web app under `examples/wasm_demo_tsp2`.
- Demo crate depends on:
  - `orx-parallel` with `wasm-web-threads2`
- Expose startup initializer in wasm boundary layer (same pattern):
  - `init_parallel_runtime(...)` calling `orx_parallel::init_thread_pool(...)`
- Frontend startup sequence remains:
  1. `await init()`
  2. `await init_parallel_runtime(...)`
  3. enable parallel actions

## Phase 5: Tests and Validation

## 1. Compile-time matrix

- Native path unchanged.
- Legacy wasm path (`wasm-web-threads`) unchanged.
- New wasm path (`wasm-web-threads2`) builds without rayon dependencies.

## 2. Behavior smoke tests

- panic/fail path before initialization
- success path after initialization
- thread-cap behavior and `num_threads` interactions
- `DefaultPool` routing under `wasm-web-threads2`

## 3. Runtime demo validation

- Run `wasm_demo_tsp2` under COOP/COEP headers.
- Validate that parallel runs complete and UI behavior matches expectations.

## Phase 6: Documentation and Rollout

- Add docs page for new backend (or extend existing wasm docs with dual-backend section).
- Clearly mark `wasm-web-threads` as legacy/compat backend once `wasm-web-threads2` stabilizes.
- Keep both features for at least one transition release.

## Suggested Work Order

1. Feature and cfg wiring
- `Cargo.toml`
- `src/pool/pool_impl/mod.rs`
- `src/pool/mod.rs`
- `src/pool/new_pool.rs`

2. Scaffold new backend
- `src/pool/pool_impl/wasm_web2.rs`
- internal helper modules as needed

3. Add tests
- wasm smoke tests for init contract and pool behavior

4. Add new demo
- `examples/wasm_demo_tsp2`

5. Docs update
- startup contract, build flags, feature selection guidance

## Risks and Mitigations

## Risk 1: Worker runtime bootstrap complexity

- Mitigation: keep first version minimal, copy only essential design patterns from existing proven crates, iterate.

## Risk 2: Bundler/runtime variations

- Mitigation: first target same `wasm-pack --target web` path used by existing demo, then expand compatibility.

## Risk 3: Feature interaction ambiguity

- Mitigation: explicit cfg guards and compile-time conflict checks.

## Risk 4: Behavioral regressions in scheduling semantics

- Mitigation: add focused smoke tests for initialization, scope completion, panic propagation, and thread caps.

## Acceptance Criteria

- `wasm-web-threads2` works on wasm web-thread builds without `rayon-core`/`wasm-bindgen-rayon`.
- `DefaultPool` resolves to new wasm pool when `wasm-web-threads2` is active.
- Startup and usage model remain similar to current `wasm_demo_tsp`.
- `wasm_demo_tsp2` demonstrates equivalent user-facing behavior.
- Existing `wasm-web-threads` path remains functional during transition.

## PR-Ready Task Breakdown

This section converts the plan into mergeable PR slices with explicit boundaries.

## Branching and Merge Strategy

- Base branch: `main`
- Feature branch prefix: `feat/wasm-web-threads2-*`
- Merge order: PR-1 -> PR-2 -> PR-3 -> PR-4 -> PR-5 -> PR-6
- Keep each PR independently reviewable and green in CI.

## PR-1: Feature Wiring and Public Surface Scaffold

### Objective

Introduce feature flags and exports for the new backend without implementing runtime logic yet.

### Scope

- Add `wasm-web-threads2` feature in `Cargo.toml`.
- Add module gates and re-exports for `WasmWebPool2` and `init_thread_pool`.
- Add `Pool::wasm_web2(...)` factory method.
- Add deterministic feature conflict guard if both wasm backends are enabled.

### Files

- `Cargo.toml`
- `src/pool/mod.rs`
- `src/pool/pool_impl/mod.rs`
- `src/pool/new_pool.rs`
- `src/pool/pool_impl/wasm_web2.rs` (stub only; compile placeholder)

### Validation

- `cargo check --features std`
- `cargo check --target wasm32-unknown-unknown --features wasm-web-threads2`
- `cargo check --target wasm32-unknown-unknown --features wasm-web-threads,wasm-web-threads2` should fail with intended compile-time conflict message.

### Definition of Done

- New feature is visible and wired.
- Build succeeds for `wasm-web-threads2` target path.
- No runtime behavior change yet.

## PR-2: Runtime Bootstrap API (Init Contract)

### Objective

Implement the initialization contract for wasm-web-threads2, including one-time startup and readiness checks.

### Scope

- Implement `init_thread_pool(num_threads) -> js_sys::Promise` for new backend.
- Add one-time init state and fail-fast readiness checks.
- Preserve atomics-required panic/diagnostic behavior.

### Files

- `src/pool/pool_impl/wasm_web2.rs`
- optional helper modules under `src/pool/pool_impl/` if needed
- docs updates for init lifecycle notes

### Validation

- compile checks from PR-1
- targeted wasm smoke compile/tests for init-uninitialized and init-success paths (or no-run compile smoke if runtime environment is unavailable in CI)

### Definition of Done

- Startup API exists and is callable from wasm boundary.
- Uninitialized usage has clear fail-fast behavior.

## PR-3: WasmWebPool2 ParThreadPool Execution

### Objective

Implement scoped execution semantics for the new pool backend.

### Scope

- Implement `ParThreadPool` for `WasmWebPool2` and `&WasmWebPool2`.
- Ensure `max_num_threads` semantics align with `NumThreads::Auto` and `NumThreads::Max(n)`.
- Ensure compatibility with existing runner expectations (scoped task scheduling and completion semantics).

### Files

- `src/pool/pool_impl/wasm_web2.rs`
- potentially shared runtime helper module(s)

### Validation

- existing wasm check commands
- representative parallel iterator smoke tests under wasm target:
  - `map + reduce`
  - `find/first` short-circuit paths
  - error path before init

### Definition of Done

- New backend can run parallel tasks with correct scope completion behavior.
- Thread cap logic behaves as expected.

## PR-4: DefaultPool Routing and Compatibility Guarantees

### Objective

Switch wasm default pool routing to the new backend under `wasm-web-threads2` and verify compatibility.

### Scope

- Make `DefaultPool` resolve to `WasmWebPool2` when `wasm-web-threads2` is active on wasm32.
- Confirm legacy route remains unchanged under `wasm-web-threads`.
- Add/adjust tests that assert feature-based pool selection behavior.

### Files

- `src/pool/mod.rs`
- tests related to cfg/feature routing

### Validation

- compile matrix:
  - native default features
  - wasm with `wasm-web-threads`
  - wasm with `wasm-web-threads2`
- smoke assertions for default pool behavior under each configuration

### Definition of Done

- New backend is default only when intended.
- No regression for legacy backend path.

## PR-5: New Demo wasm_demo_tsp2

### Objective

Provide a user-facing demonstration that mirrors existing demo ergonomics with the new backend.

### Scope

- Add `examples/wasm_demo_tsp2` with crate + web structure.
- Use `orx-parallel` with `wasm-web-threads2` in demo crate.
- Expose `init_parallel_runtime(...)` and parallel/sequential run functions following existing demo style.
- Keep frontend startup sequence aligned with current demo.

### Files

- `examples/wasm_demo_tsp2/**`

### Validation

- demo crate compiles for wasm target
- web app starts and performs parallel run after awaited init
- visual/interaction parity checks against existing demo behavior

### Definition of Done

- Demo clearly validates plan goals and API ergonomics.

## PR-6: Docs, Migration Notes, and Final Hardening

### Objective

Finalize operator guidance, migration path, and verification notes.

### Scope

- Update wasm docs to explain dual backend features and when to use each.
- Add migration guidance from `wasm-web-threads` to `wasm-web-threads2`.
- Document known limitations and troubleshooting for new backend.
- Add release-note draft bullets.

### Files

- `docs/wasm.md`
- `docs/wasm_web_threads.md` (or split into legacy/new pages)
- `README.md` sections as needed
- `docs/wasm-plan-b.md` (mark implementation status)

### Validation

- docs examples compile or are at least command-validated where possible
- commands and feature names are consistent with code

### Definition of Done

- Users can adopt new backend from docs without code archaeology.

## Global Checklist (Tracking)

- [ ] PR-1 merged
- [ ] PR-2 merged
- [ ] PR-3 merged
- [ ] PR-4 merged
- [ ] PR-5 merged
- [ ] PR-6 merged
- [ ] CI green for native + wasm legacy + wasm2 paths
- [ ] Demo wasm_demo_tsp2 validated in browser runtime

## Reviewer Checklist Per PR

- [ ] Feature gates are minimal and correct.
- [ ] No accidental dependency on `rayon-core` or `wasm-bindgen-rayon` under `wasm-web-threads2`.
- [ ] Public API changes are documented.
- [ ] Error messages are actionable.
- [ ] Test/compile commands are included in PR description.

## Suggested PR Template Snippet

Use this in each PR description.

```md
## Summary
-

## Scope
-

## Out of Scope
-

## Validation
- [ ] cargo check ...
- [ ] wasm target check ...
- [ ] smoke test ...

## Risks
-

## Follow-ups
-
```
