use crate::parameters::non_zero_or_one;
use crate::pool::ParThreadPool;
use core::num::NonZeroUsize;
use rayon_core::ThreadPool;

impl ParThreadPool for ThreadPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon_core::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
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
        self.scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        non_zero_or_one(self.current_num_threads())
    }
}

impl ParThreadPool for &rayon_core::ThreadPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon_core::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
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
        self.scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        non_zero_or_one(self.current_num_threads())
    }
}

// 3. rayon
#[cfg(all(
    feature = "std",
    feature = "persistent-pool-rayon",
    not(all(feature = "wasm", target_arch = "wasm32")),
))]
pub fn build_default_rayon_thread_pool() -> rayon_core::ThreadPool {
    let num_threads = crate::pool::env::max_num_threads_by_env_and_resource();
    rayon_core::ThreadPoolBuilder::new()
        .num_threads(num_threads.into())
        .build()
        .expect("failed to build rayon-core thread pool")
}
