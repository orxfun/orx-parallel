# Threading Model: Pool Configuration and Thread Count Decision

This document explains how orx-parallel determines the number of threads to use for parallel computations through three layers of configuration.

## Overview

The actual number of threads used in a parallel computation is determined by the following hierarchy:

1. **Pool Layer** (`ParThreadPool::max_num_threads()`) - Maximum threads available in the thread pool
2. **Environment Layer** (`ORX_PARALLEL_MAX_NUM_THREADS`) - Global limit from environment variable
3. **Computation Layer** (`.num_threads()` method) - Per-computation configuration

The actual thread count is the minimum of these three constraints.

---

## Layer 1: Pool Layer (`ParThreadPool`)

### The `ParThreadPool` Trait

`ParThreadPool` is the core abstraction defining a parallel execution environment. Any type implementing this trait can be used as a thread pool for parallel computations.

**Required Methods:**
- `max_num_threads(&self) -> NonZeroUsize` - Returns the maximum number of threads available in the pool
- `run_in_scope<W>(scope: &ScopeRef, work: W)` - Executes work within a scope
- `scoped_computation<F>(f: F)` - Executes a scoped computation

**Provided Method:**
- `max_num_threads_for_computation(&self, params: Params, size_hint: (usize, Option<usize>)) -> usize` - Calculates the actual thread count considering computation parameters and input size

### Thread Count Calculation in `max_num_threads_for_computation()`

This method implements the core decision logic:

```rust
fn max_num_threads_for_computation(
    &self,
    params: Params,
    size_hint: (usize, Option<usize>),
) -> usize {
    let ava = self.max_num_threads();  // Available threads in pool
    
    let req = match (size_hint.1, params.num_threads) {
        // Input has known upper bound, request is auto => min(input_len, MaxUsize)
        (Some(len_ub), NumThreads::Auto) => NonZeroUsize::new(len_ub.max(1)).expect(">0"),
        
        // Input has known upper bound, request is Max(n) => min(input_len, n)
        (Some(len_ub), NumThreads::Max(nt)) => {
            NonZeroUsize::new(len_ub.max(1)).expect(">0").min(nt)
        }
        
        // Input size unknown, request is auto => MaxUsize (no limit)
        (None, NumThreads::Auto) => NonZeroUsize::MAX,
        
        // Input size unknown, request is Max(n) => n
        (None, NumThreads::Max(nt)) => nt,
    };
    
    // Final decision: min(requested, available)
    core::cmp::min(req, ava).into()
}
```

### Built-in Pool Implementations

#### 1. `OncePool` - Default Pool for Standard Features

Located in `src/pool/pool_impl/once.rs`.

**What it is:** A lightweight virtual pool that spawns threads on-demand for a single computation, then releases them.

**Characteristics:**
- Default pool when `std` feature is enabled
- Not an actual persistent thread pool
- Threads are spawned just before computation and released after
- Useful for reducing overhead of pool creation when a persistent pool isn't needed

**Construction with Thread Count Logic:**

```rust
impl OncePool {
    pub fn new(num_threads: impl Into<NumThreads>) -> Self {
        let num_threads = match num_threads.into() {
            // Auto: use all threads available after env constraints
            NumThreads::Auto => max_num_threads_by_env_and_resource(),
            
            // Max(n): cap at n, but also apply env constraints
            NumThreads::Max(n) => max_num_threads_by_env_and_resource().min(n),
        };
        Self { num_threads }
    }
}
```

#### 2. `BasicPool` - Persistent Thread Pool

Located in `src/pool/pool_impl/basic.rs`.

**What it is:** A persistent thread pool that maintains worker threads across multiple computations.

**Characteristics:**
- Threads remain alive across multiple computations
- Reduces overhead when running multiple parallel computations
- Suitable for applications with many parallel tasks

#### 3. Rayon Pool - External Integration

```rust
impl Pool {
    #[cfg(feature = "rayon-core")]
    pub fn rayon(
        num_threads: impl Into<NumThreads>,
    ) -> Result<rayon_core::ThreadPool, rayon_core::ThreadPoolBuildError>
    // Creates a Rayon ThreadPool integrated with orx-parallel
}
```

### Using `Pool` Factory

The `Pool` struct provides factory methods to create thread pools:

```rust
// Create a OncePool with automatic thread detection
let pool = Pool::once(NumThreads::Auto);

// Create a OncePool capped at 4 threads
let pool = Pool::once(4);  // Converted from usize via From impl

// Create a BasicPool for persistent usage
let pool = Pool::basic(8);

// Create a Rayon pool (rayon-core feature required)
let pool = Pool::rayon(NumThreads::Auto)?;
```

---

## Layer 2: Environment Layer (`ORX_PARALLEL_MAX_NUM_THREADS`)

### Environment Variable Configuration

The `ORX_PARALLEL_MAX_NUM_THREADS` environment variable provides a **global hard limit** on the maximum number of threads any parallel computation can use.

**Location:** `src/pool/env.rs`

**Functions:**
- `max_num_threads_by_env_variable() -> Option<NonZeroUsize>` - Reads and parses the environment variable
- `max_num_threads_by_env_and_resource() -> NonZeroUsize` - Combines env limit with system resources

### Parsing Logic

```rust
pub fn max_num_threads_by_env_variable() -> Option<NonZeroUsize> {
    match std::env::var("ORX_PARALLEL_MAX_NUM_THREADS") {
        Ok(s) => match s.parse::<usize>() {
            Ok(x) => NonZeroUsize::new(x),  // None if 0, Some(x) if positive
            Err(_) => None,                  // Not a valid number, ignored
        },
        Err(_) => None,  // Not set, no constraint
    }
}
```

**Behavior:**
- `ORX_PARALLEL_MAX_NUM_THREADS=0` → Unset, no constraint
- `ORX_PARALLEL_MAX_NUM_THREADS=4` → Hard limit of 4 threads
- `ORX_PARALLEL_MAX_NUM_THREADS=invalid` → Ignored, treated as unset

### Resource-Aware Calculation

```rust
pub fn max_num_threads_by_env_and_resource() -> NonZeroUsize {
    let env_max = max_num_threads_by_env_variable();
    let ava_max = std::thread::available_parallelism().ok();  // System CPUs
    
    match (env_max, ava_max) {
        // Both set: take the smaller
        (Some(env), Some(ava)) => if env < ava { env } else { ava },
        
        // Only env set: use env
        (Some(env), None) => env,
        
        // Only system knows: use system
        (None, Some(ava)) => ava,
        
        // Neither set: use default (8)
        (None, None) => NonZeroUsize::new(8).expect(">0"),
    }
}
```

### Examples

```bash
# No constraint - uses system CPU count (or 8 if unavailable)
cargo run --release --example my_example

# Hard limit to 4 threads
ORX_PARALLEL_MAX_NUM_THREADS=4 cargo run --release --example my_example

# Hard limit to 1 thread (sequential execution)
ORX_PARALLEL_MAX_NUM_THREADS=1 cargo run --release --example my_example
```

---

## Layer 3: Computation Layer - `.num_threads()` Method

### Per-Computation Thread Configuration

Every parallel iterator builder offers a `.num_threads()` method to configure thread count for that specific computation:

```rust
fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;
```

**Example Usage:**

```rust
use orx_parallel::*;

// Use all available threads
let result: Vec<_> = (0..1000)
    .into_par()
    .map(|x| x * 2)
    .num_threads(NumThreads::Auto)  // Default if not specified
    .collect();

// Cap at 4 threads
let result: Vec<_> = (0..1000)
    .into_par()
    .map(|x| x * 2)
    .num_threads(4)  // Converted via From<usize> impl
    .collect();

// Force sequential execution
let result: Vec<_> = (0..1000)
    .into_par()
    .map(|x| x * 2)
    .num_threads(NumThreads::sequential())  // or .num_threads(1)
    .collect();
```

### `NumThreads` Type

Located in `src/parameters/num_threads.rs`.

**Variants:**
- `NumThreads::Auto` (default) - Uses all available threads, spawns only as needed
- `NumThreads::Max(NonZeroUsize)` - Hard cap on thread count

**Conversion from `usize`:**
- `0` → `NumThreads::Auto`
- `n > 0` → `NumThreads::Max(n)`

```rust
let nt: NumThreads = 4.into();  // Converts to NumThreads::Max(4)
let nt: NumThreads = 0.into();  // Converts to NumThreads::Auto
```

---

## Complete Thread Count Decision Flow

### Step 1: Pool Creation (Layer 1 + Layer 2)

When a pool is created, it combines:
- The requested thread count from pool construction
- The `ORX_PARALLEL_MAX_NUM_THREADS` environment variable constraint

```
OncePool::new(num_threads) →
  if Auto: max_num_threads_by_env_and_resource()
  if Max(n): max_num_threads_by_env_and_resource().min(n)
```

**Result:** The pool's `max_num_threads()` reflects all constraints up to this point.

### Step 2: Computation Submission (Layer 3)

When a computation starts, it considers:
- The pool's `max_num_threads()`
- The computation's `.num_threads()` setting (Layer 3)
- The input size hint

```
max_num_threads_for_computation(params, size_hint) →
  let available = self.max_num_threads()  // From pool (includes env)
  let requested = params.num_threads       // From .num_threads()
  
  consider input size:
    if input_size_known:
      requested = min(requested, input_size)
  
  return min(requested, available)
```

### Example: Complete Flow

```rust
use orx_parallel::*;

// Pool setup (Step 1)
// ORX_PARALLEL_MAX_NUM_THREADS=4 is set
let pool = Pool::once(8);  // Request 8, but env limits to 4
                           // pool.max_num_threads() == 4

// Computation setup (Step 2)
let result: Vec<_> = (0..100)
    .into_par()
    .map(|x| x * 2)
    .pool(pool)
    .num_threads(6)        // Request 6 for this computation
    .collect();

// Decision logic:
// - Pool has 4 (from 8 capped by env)
// - Computation requests 6
// - Input has 100 elements
// Result: min(min(6, 100), 4) = 4 threads used
```

### Precedence Summary

From highest to lowest precedence:
1. **Smallest constraint wins**: The minimum of all three layers
2. **Pool limit** (`pool.max_num_threads()`) - Set at pool creation, includes env constraint
3. **Computation limit** (`.num_threads()`) - Set per-computation
4. **Input size** (known upper bound) - Cannot spawn more threads than input elements
5. **Environment** (applies at pool creation time via `max_num_threads_by_env_and_resource()`)

---

## Best Practices

### 1. Default Configuration

For most applications, use defaults:
```rust
let result = (0..10000)
    .into_par()
    .map(|x| process(x))
    .collect();
```
- `NumThreads::Auto` is the default
- System will spawn threads only as needed
- Optimal for heterogeneous workloads

### 2. Resource-Constrained Environments

Set a hard limit via environment variable:
```bash
# Docker container with 2 CPU cores
ORX_PARALLEL_MAX_NUM_THREADS=2 ./my_app
```

### 3. Performance Tuning

When benchmarks show benefit, set per-computation limits:
```rust
let result = (0..1_000_000)
    .into_par()
    .map(|x| expensive_operation(x))
    .num_threads(8)  // Benchmarks show 8 is optimal
    .collect();
```

### 4. Sequential Fallback

Use sequential execution for testing or debugging:
```rust
let result = (0..1000)
    .into_par()
    .map(|x| x * 2)
    .num_threads(1)  // Identical to sequential Iterator
    .collect();
```

### 5. Persistent Pools for Multiple Computations

Use `BasicPool` or `Pool::rayon()` when running many computations:
```rust
let pool = Pool::basic(4);

for computation in computations {
    let result = (0..10000)
        .into_par()
        .map(|x| process(computation, x))
        .pool(pool)
        .collect();
}
```

---

## Diagnostics and Observation

To understand thread usage, enable diagnostics:

```rust
use orx_parallel::*;

let result = (0..10000)
    .into_par()
    .map(|x| x * 2)
    .runner_with_diagnostics()  // Track actual thread usage
    .collect();
```

See the `diagnostics` example for detailed metrics collection.

---

## See Also

TODO: 