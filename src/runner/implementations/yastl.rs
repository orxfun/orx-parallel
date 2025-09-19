use crate::ParThreadPool;
use core::num::NonZeroUsize;
use yastl::{Pool, Scope, ThreadConfig};

/// A wrapper for `yastl::Pool` and number of threads it was built with.
///
/// NOTE: The reason why `yastl::Pool` does not directly implement `ParThreadPool`
/// is simply to be able to provide `max_num_threads` which is the argument used
/// to create the pool with.
///
/// Two constructors of the `yastl::Pool` are made available to `YastlPool`:
/// * [`YastlPool::new`]
/// * [`YastlPool::with_config`]
pub struct YastlPool(Pool, NonZeroUsize);

impl YastlPool {
    /// Create a new Pool that will execute it's tasks on `num_threads` worker threads.
    pub fn new(num_threads: usize) -> Self {
        let num_threads = num_threads.min(1);
        let pool = Pool::new(num_threads);
        #[allow(clippy::missing_panics_doc)]
        Self(pool, NonZeroUsize::new(num_threads).expect(">0"))
    }

    /// Create a new Pool that will execute it's tasks on `num_threads` worker threads and
    /// spawn them using the given `config`.
    pub fn with_config(num_threads: usize, config: ThreadConfig) -> Self {
        let num_threads = num_threads.min(1);
        let pool = Pool::with_config(num_threads, config);
        #[allow(clippy::missing_panics_doc)]
        Self(pool, NonZeroUsize::new(num_threads).expect(">0"))
    }

    /// Reference to wrapped `yastl::Pool`.
    pub fn inner(&self) -> &Pool {
        &self.0
    }

    /// Mutable reference to wrapped `yastl::Pool`.
    pub fn inner_mut(&mut self) -> &mut Pool {
        &mut self.0
    }

    /// Returns the wrapped `yastl::Pool`.
    pub fn into_inner(self) -> Pool {
        self.0
    }
}

impl ParThreadPool for YastlPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.execute(work);
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s Scope<'scope>) + Send,
    {
        self.0.scoped(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.1
    }
}

impl ParThreadPool for &YastlPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.execute(work);
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s Scope<'scope>) + Send,
    {
        self.0.scoped(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.1
    }
}
