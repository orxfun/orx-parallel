use crate::NumThreads;
use crate::pool::ParThreadPool;
use core::num::NonZeroUsize;

/// wasm web-thread pool adapter backed by Rayon's global runtime.
///
/// This pool is available only on `wasm32` with `wasm-web-threads` feature.
/// It schedules scoped jobs on Rayon's global scope.
#[derive(Clone, Copy, Debug)]
pub struct WasmWebPool {
    max_num_threads: NonZeroUsize,
}

impl Default for WasmWebPool {
    fn default() -> Self {
        Self::new(NumThreads::Auto)
    }
}

impl WasmWebPool {
    /// Creates a new wasm web-thread pool adapter.
    pub fn new(num_threads: impl Into<NumThreads>) -> Self {
        let rayon_max = NonZeroUsize::new(rayon_core::max_num_threads().max(1)).expect(">0");
        let max_num_threads = match num_threads.into() {
            NumThreads::Auto => rayon_max,
            NumThreads::Max(n) => n.min(rayon_max),
        };

        Self { max_num_threads }
    }
}

impl ParThreadPool for WasmWebPool {
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
        rayon_core::scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }
}

impl ParThreadPool for &WasmWebPool {
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
        rayon_core::scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }
}
