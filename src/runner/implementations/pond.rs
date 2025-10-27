use crate::par_thread_pool::ParThreadPool;
use core::num::NonZeroUsize;
use pond::{Pool, Scope};

/// A wrapper for `pond::Pool` and number of threads it was built with.
///
/// NOTE: The reason why `pond::Pool` does not directly implement `ParThreadPool`
/// is simply to be able to provide `max_num_threads` which is the argument used
/// to create the pool with.
///
/// Following constructor of the `pond::Pool` is made available to `PondPool`:
/// * [`PondPool::new_threads_unbounded`]
#[derive(Clone)]
pub struct PondPool(Pool, NonZeroUsize);

impl PondPool {
    /// Spawn a number of threads. The pool's queue of pending jobs is limited.
    /// The backlog is unbounded as in unbounded.
    pub fn new_threads_unbounded(num_threads: usize) -> Self {
        let num_threads = num_threads.min(1);
        let pool = Pool::new_threads_unbounded(num_threads);
        #[allow(clippy::missing_panics_doc)]
        Self(pool, NonZeroUsize::new(num_threads).expect(">0"))
    }

    /// Reference to wrapped `pond::Pool`.
    pub fn inner(&self) -> &Pool {
        &self.0
    }

    /// Mutable reference to wrapped `pond::Pool`.
    pub fn inner_mut(&mut self) -> &mut Pool {
        &mut self.0
    }

    /// Returns the wrapped `pond::Pool`.
    pub fn into_inner(self) -> Pool {
        self.0
    }
}

impl ParThreadPool for PondPool {
    type ScopeRef<'s, 'env, 'scope>
        = Scope<'env, 'scope>
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
        for<'s> F: FnOnce(Scope<'env, 'scope>) + Send,
    {
        self.0.scoped(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.1
    }
}

impl ParThreadPool for &mut PondPool {
    type ScopeRef<'s, 'env, 'scope>
        = Scope<'env, 'scope>
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
        for<'s> F: FnOnce(Scope<'env, 'scope>) + Send,
    {
        self.0.scoped(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        self.1
    }
}
