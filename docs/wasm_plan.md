# WASM Enablement Plan for orx-parallel

## Objective
Enable orx-parallel to run in web wasm builds with real parallelism via Web Workers, while preserving current native behavior.

## Current State (Verified)
- The crate builds for `wasm32-unknown-unknown` in CI, but this does not yet represent a web-threaded runtime path.
- Pool abstractions and built-in pools are currently designed around synchronous, native threading semantics.
- Rayon integration exists today via `Pool::rayon(...)` when `rayon-core` is enabled.
- No wasm-specific runtime docs or wasm-thread setup currently exist in repository docs.

## Constraints From wasm-bindgen-rayon
- Threaded wasm on web requires async initialization (`initThreadPool`) from JavaScript.
- Uses worker-backed Rayon global initialization flow.
- Requires wasm thread build setup (nightly + atomics/bulk-memory + std rebuild settings).
- Requires SharedArrayBuffer prerequisites (COOP/COEP cross-origin isolation).

## Rollout Strategy
Deliver in small, reviewable PRs with strict backward compatibility for native targets.

## Implementation Status

- PR1 completed.
- PR2 completed.
- PR3 completed.
- PR4 completed.
- PR5 completed.
- PR6 completed.

## PR Roadmap

### PR1: Feature Gating and Dependency Skeleton
Goal:
- Introduce wasm-thread support as explicit opt-in without changing existing behavior.

Scope:
- Add a dedicated wasm-thread feature flag.
- Add wasm32 target-specific dependencies behind that feature.
- Keep existing `std` and `rayon-core` behavior unchanged.

Acceptance Criteria:
- Native stable builds remain unchanged.
- Existing wasm32 build/check matrix still passes.
- Enabling the new feature resolves dependencies and compiles (even before runtime integration).

Risks:
- Feature interaction complexity with existing matrix (`default`, `--all-features`, `--no-default-features`).

---

### PR2: Wasm Pool Adapter (Core Integration)
Goal:
- Provide a wasm-compatible `ParThreadPool` path backed by Rayon global runtime.

Scope:
- Add wasm-only pool implementation module.
- Add a `Pool` constructor for wasm threaded mode.
- Keep native pool implementations untouched.

Acceptance Criteria:
- New wasm pool compiles for wasm target with threaded feature enabled.
- No regressions in existing pool implementations and APIs.
- Native behavior remains unchanged.

Risks:
- Mapping current scope-based API expectations to wasm runtime assumptions.

---

### PR3: Initialization Contract (`initThreadPool` Exposure)
Goal:
- Define and expose deterministic initialization semantics for consumers.

Scope:
- Re-export or wrap `init_thread_pool` behind wasm feature.
- Document required call order: module init -> thread pool init -> parallel API calls.
- Enforce behavior when pool is not initialized.

Acceptance Criteria:
- Downstream JS can initialize and use parallel APIs successfully.
- Uninitialized usage has deterministic behavior (error or explicit fallback per decision).
- Runtime messages are actionable.

Risks:
- Confusion around async init timing if call order is not enforced clearly.

---

### PR4: Build Toolchain and CI for Threaded Wasm
Goal:
- Make threaded wasm build reproducible and continuously validated.

Scope:
- Add documented pinned nightly workflow.
- Add wasm thread flags and `build-std` guidance.
- Add dedicated CI job for threaded wasm configuration.

Acceptance Criteria:
- CI validates both baseline wasm compile path and threaded wasm path.
- Build instructions reproduce successfully on clean setup.
- Failures clearly indicate whether they are baseline wasm or threaded wasm issues.

Risks:
- Toolchain drift on nightly and build-std behavior.

---

### PR5: Runtime Validation Example and Smoke Coverage
Goal:
- Prove end-to-end runtime path with minimal user-facing example.

Scope:
- Add minimal wasm example showing initialization and one representative parallel operation.
- Add smoke checks for success path and missing-init behavior.

Acceptance Criteria:
- Example reliably demonstrates required initialization and execution order.
- Smoke checks cover initialization success and failure/fallback branch.
- No regressions in existing test suite.

Risks:
- Browser/runtime variability in local and CI environments.

---

### PR6: Documentation and Migration Guidance
Goal:
- Make wasm adoption straightforward and safe.

Scope:
- Add dedicated wasm docs page/section.
- Add browser prerequisites and header requirements.
- Add dual-build strategy guidance with runtime feature detection.
- Add troubleshooting section.

Acceptance Criteria:
- New users can follow docs to run threaded wasm example.
- Existing native users have zero migration burden.
- Runtime behavior and docs are fully aligned.

Risks:
- Documentation drift if runtime behavior changes after docs merge.

## Definition of Done (Cross-PR)
- Native Linux/macOS/Windows behavior remains unchanged.
- Baseline wasm32 builds still work as before.
- Threaded wasm mode works end-to-end with documented init sequence.
- CI has explicit coverage for both wasm modes.
- Public docs describe behavior, requirements, and failure modes accurately.

## Design Decisions

### 1) Wasm execution model: global-only pool or custom pool support?
Options:
- A. Global-only wasm pool (Rayon global runtime via wasm-bindgen-rayon)
- B. User-supplied dedicated pools on wasm

Recommendation:
- A first (global-only)

Rationale:
- Aligns with wasm-bindgen-rayon initialization model.
- Lowest complexity for first production release.
- Minimizes lifecycle and API surface risk.

Future extension:
- Revisit custom wasm pool support after stable baseline.

---

### 2) Missing initialization behavior: fail-fast or fallback?
Options:
- A. Fail-fast with explicit error
- B. Silent sequential fallback
- C. Configurable policy

Recommendation:
- A (fail-fast) by default

Rationale:
- Avoids silent performance regressions.
- Easier diagnosis in production.
- Enables explicit fallback policy later if desired.

---

### 3) Feature model: dedicated wasm feature or implicit via rayon-core?
Options:
- A. Explicit wasm-thread feature
- B. Implicit activation through existing `rayon-core` combinations

Recommendation:
- A (explicit feature)

Rationale:
- Clear intent for users.
- Cleaner dependency boundaries.
- Lower accidental activation risk.

---

### 4) Default pool behavior on wasm when threaded feature is enabled
Options:
- A. Keep current default behavior unless user opts in
- B. Auto-switch default pool on wasm when feature is enabled

Recommendation:
- A initially

Rationale:
- Avoid surprising behavior changes.
- Keeps rollout conservative and debuggable.
- Auto-switch can be evaluated in a later major release.

## Proposed Decision Set for Initial Release
- Global-only wasm pool.
- Fail-fast on missing initialization.
- Explicit wasm-thread feature.
- No automatic default-pool switch on wasm.

## Suggested Milestone Ordering
1. PR1 + PR2
2. PR3
3. PR4
4. PR5
5. PR6

This ordering establishes compile-time foundations first, then runtime contract, then validation and docs.
