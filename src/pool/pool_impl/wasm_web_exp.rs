use crate::NumThreads;
use crate::pool::ParThreadPool;
use core::num::NonZeroUsize;
use core::sync::atomic::{AtomicBool, Ordering};

static WASM_WEB_THREAD_POOL_INIT_CALLED: AtomicBool = AtomicBool::new(false);

/// Initializes the worker-backed Rayon thread pool for wasm web builds.
///
/// This must be called (and awaited from JavaScript) before running parallel
/// computations with [`WasmWebPoolExp`].
///
/// `num_threads = 0` is treated as an automatic setting and initializes the
/// pool with `rayon_core::max_num_threads().max(1)`.
#[cfg(target_feature = "atomics")]
pub fn init_thread_pool(num_threads: usize) -> js_sys::Promise {
    let num_threads = match num_threads {
        0 => rayon_core::max_num_threads().max(1),
        n => n,
    };
    WASM_WEB_THREAD_POOL_INIT_CALLED.store(true, Ordering::SeqCst);
    wasm_bindgen_rayon::init_thread_pool(num_threads)
}

fn assert_wasm_thread_pool_initialized() {
    const {
        assert!(
            cfg!(target_feature = "atomics"),
            "Wasm web threading requires atomics-enabled wasm build flags; see docs/wasm.md."
        )
    }

    assert!(
        !WASM_WEB_THREAD_POOL_INIT_CALLED.load(Ordering::SeqCst),
        "Wasm web thread pool is not initialized. Call and await init_thread_pool(...) before running parallel computations."
    );
}

/// wasm web-thread pool adapter backed by Rayon's global runtime.
///
/// This pool is available only on `wasm32` with `wasm-experimental` feature.
/// It schedules scoped jobs on Rayon's global scope.
#[derive(Clone, Copy, Debug)]
pub struct WasmWebPoolExp {
    max_num_threads: NonZeroUsize,
}

impl Default for WasmWebPoolExp {
    fn default() -> Self {
        Self::new(NumThreads::Auto)
    }
}

impl WasmWebPoolExp {
    /// Creates a new wasm web-thread pool adapter.
    #[allow(clippy::missing_panics_doc)]
    pub fn new(num_threads: impl Into<NumThreads>) -> Self {
        let rayon_max = NonZeroUsize::new(rayon_core::max_num_threads().max(1)).expect(">0");
        let max_num_threads = match num_threads.into() {
            NumThreads::Auto => rayon_max,
            NumThreads::Max(n) => n.min(rayon_max),
        };

        Self { max_num_threads }
    }
}

impl ParThreadPool for WasmWebPoolExp {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon_core::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.spawn(move |_| work());
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s rayon_core::Scope<'scope>) + Send,
    {
        assert_wasm_thread_pool_initialized();
        rayon_core::scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }
}

impl ParThreadPool for &WasmWebPoolExp {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon_core::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.spawn(move |_| work());
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s rayon_core::Scope<'scope>) + Send,
    {
        assert_wasm_thread_pool_initialized();
        rayon_core::scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }
}
