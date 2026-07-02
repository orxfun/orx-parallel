use crate::NumThreads;
use crate::pool::ParThreadPool;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

const WASM_WEB2_THREAD_POOL_UNINITIALIZED: u8 = 0;
const WASM_WEB2_THREAD_POOL_INITIALIZED: u8 = 1;

#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
static WASM_WEB2_THREAD_POOL_STATE: AtomicU8 = AtomicU8::new(WASM_WEB2_THREAD_POOL_UNINITIALIZED);
#[cfg_attr(not(target_feature = "atomics"), allow(dead_code))]
static WASM_WEB2_THREAD_POOL_NUM_THREADS: AtomicUsize = AtomicUsize::new(0);

/// Initializes the worker-backed wasm thread runtime for `WasmWebPool2`.
///
/// This establishes the runtime init contract for the new wasm backend.
#[cfg(target_feature = "atomics")]
pub fn init_thread_pool(num_threads: usize) -> js_sys::Promise {
    let num_threads = num_threads.max(1);

    match WASM_WEB2_THREAD_POOL_STATE.compare_exchange(
        WASM_WEB2_THREAD_POOL_UNINITIALIZED,
        WASM_WEB2_THREAD_POOL_INITIALIZED,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(_) => {
            WASM_WEB2_THREAD_POOL_NUM_THREADS.store(num_threads, Ordering::SeqCst);
            js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED)
        }
        Err(WASM_WEB2_THREAD_POOL_INITIALIZED) => {
            let configured_threads = WASM_WEB2_THREAD_POOL_NUM_THREADS.load(Ordering::SeqCst);

            if configured_threads == num_threads {
                js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED)
            } else {
                js_sys::Promise::reject(&wasm_bindgen::JsValue::from_str(&format!(
                    "init_thread_pool was already called with {configured_threads} threads; refusing to reinitialize with {num_threads} threads"
                )))
            }
        }
        Err(_) => unreachable!("invalid wasm-web-threads2 init state"),
    }
}

fn assert_wasm_thread_pool_initialized() {
    if !cfg!(target_feature = "atomics") {
        panic!(
            "Wasm web threading requires atomics-enabled wasm build flags; see docs/wasm-plan-b.md."
        );
    }

    if WASM_WEB2_THREAD_POOL_STATE.load(Ordering::SeqCst) != WASM_WEB2_THREAD_POOL_INITIALIZED {
        panic!(
            "Wasm web2 thread pool is not initialized. Call and await init_thread_pool(...) before running parallel computations."
        );
    }
}

/// wasm web-thread pool adapter for the new internal backend.
///
/// This PR-1 scaffold exposes the API surface; task execution is implemented in follow-up PRs.
#[derive(Clone, Copy, Debug)]
pub struct WasmWebPool2 {
    max_num_threads: NonZeroUsize,
}

impl Default for WasmWebPool2 {
    fn default() -> Self {
        Self::new(NumThreads::Auto)
    }
}

impl WasmWebPool2 {
    /// Creates a new wasm web-thread pool adapter.
    pub fn new(num_threads: impl Into<NumThreads>) -> Self {
        let max_num_threads = match num_threads.into() {
            NumThreads::Auto => NonZeroUsize::new(1).expect(">0"),
            NumThreads::Max(n) => n,
        };

        Self { max_num_threads }
    }
}

#[derive(Debug)]
pub struct ScopeRef2;

impl ParThreadPool for WasmWebPool2 {
    type ScopeRef<'s, 'env, 'scope>
        = &'s ScopeRef2
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(_s: &Self::ScopeRef<'s, 'env, 'scope>, _work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        panic!("WasmWebPool2::run_in_scope is not implemented yet (PR-1 scaffold).");
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, _f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef2) + Send,
    {
        assert_wasm_thread_pool_initialized();
        panic!("WasmWebPool2::scoped_computation is not implemented yet (PR-1 scaffold).");
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }
}

impl ParThreadPool for &WasmWebPool2 {
    type ScopeRef<'s, 'env, 'scope>
        = &'s ScopeRef2
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(_s: &Self::ScopeRef<'s, 'env, 'scope>, _work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        panic!("WasmWebPool2::run_in_scope is not implemented yet (PR-1 scaffold).");
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, _f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s ScopeRef2) + Send,
    {
        assert_wasm_thread_pool_initialized();
        panic!("WasmWebPool2::scoped_computation is not implemented yet (PR-1 scaffold).");
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }
}
