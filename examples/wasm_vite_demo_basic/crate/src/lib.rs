use serde::Serialize;
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
use orx_parallel::{IntoParIter, Par};

const FIXED_THREADS: u32 = 4;
const MAX_N: u32 = 93;

#[derive(Debug, Serialize)]
struct FibSumResult {
    start: u32,
    end: u32,
    sum: u64,
    threads: u32,
    elapsed_ms: f64,
}

#[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
#[wasm_bindgen]
/// Initializes the wasm thread pool with a fixed number of worker threads.
pub fn init_parallel_runtime() -> js_sys::Promise {
    orx_parallel::init_thread_pool(FIXED_THREADS as usize)
}

#[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
#[wasm_bindgen]
/// Returns an error when wasm threaded runtime is unavailable.
pub fn init_parallel_runtime() -> Result<JsValue, JsValue> {
    Err(JsValue::from_str(
        "init_parallel_runtime is only available for wasm32 + atomics builds",
    ))
}

#[wasm_bindgen]
/// Computes the sum of Fibonacci numbers in the inclusive range [start, end] in parallel.
pub fn run_fib_sum(start: u32, end: u32) -> Result<JsValue, JsValue> {
    #[cfg(not(all(target_arch = "wasm32", target_feature = "atomics")))]
    {
        let _ = (start, end);
        return Err(JsValue::from_str(
            "run_fib_sum requires wasm32 + atomics build",
        ));
    }

    #[cfg(all(target_arch = "wasm32", target_feature = "atomics"))]
    {
        if start > end {
            return Err(JsValue::from_str("start must be <= end"));
        }

        if end > MAX_N {
            return Err(JsValue::from_str("end is too large; use end <= 93"));
        }

        let started_at = js_sys::Date::now();
        let range_start = start as usize;
        let range_end_exclusive = (end as usize) + 1;

        let sum: u64 = (range_start..range_end_exclusive)
            .into_par()
            .map(|n| fibonacci(n as u32))
            .sum();

        let result = FibSumResult {
            start,
            end,
            sum,
            threads: FIXED_THREADS,
            elapsed_ms: js_sys::Date::now() - started_at,
        };

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("failed to serialize result: {e}")))
    }
}

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let next = a.saturating_add(b);
                a = b;
                b = next;
            }
            b
        }
    }
}
