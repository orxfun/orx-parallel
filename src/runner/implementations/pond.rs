use crate::{DefaultExecutor, ParThreadPool, ParallelExecutor, runner::ParallelRunner};
use core::{marker::PhantomData, num::NonZeroUsize};
use orx_self_or::SoM;
use pond::{Pool, Scope};

// POOL

/// A wrapper for `pond::Pool` and number of threads it was built with.
///
/// NOTE: The reason why `pond::Pool` does not directly implement `ParThreadPool`
/// is simply to be able to provide `max_num_threads` which is the argument used
/// to create the pool with.
///
/// Following constructor of the `pond::Pool` is made available to `PondPool`:
/// * [`PondPool::new_threads_unbounded`]
pub struct PondPool(Pool, NonZeroUsize);

impl PondPool {
    /// Spawn a number of threads. The pool's queue of pending jobs is limited.
    /// The backlog is unbounded as in unbounded.
    pub fn new_threads_unbounded(num_threads: usize) -> Self {
        let num_threads = num_threads.min(1);
        let pool = Pool::new_threads_unbounded(num_threads);
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

// RUNNER

/// Parallel runner using threads provided by pond::Pool.
pub struct RunnerWithPondPool<P, R = DefaultExecutor>
where
    R: ParallelExecutor,
    P: SoM<PondPool> + ParThreadPool,
{
    pool: P,
    runner: PhantomData<R>,
}

impl From<PondPool> for RunnerWithPondPool<PondPool, DefaultExecutor> {
    fn from(pool: PondPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<'a> From<&'a mut PondPool> for RunnerWithPondPool<&'a mut PondPool, DefaultExecutor> {
    fn from(pool: &'a mut PondPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> ParallelRunner for RunnerWithPondPool<P, R>
where
    R: ParallelExecutor,
    P: SoM<PondPool> + ParThreadPool,
{
    type Executor = R;

    type ThreadPool = P;

    fn thread_pool(&self) -> &Self::ThreadPool {
        &self.pool
    }

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool {
        &mut self.pool
    }
}
