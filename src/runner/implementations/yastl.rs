use crate::{DefaultExecutor, ParThreadPool, ParallelExecutor, runner::ParallelRunner};
use core::{marker::PhantomData, num::NonZeroUsize};
use orx_self_or::SoR;
use yastl::{Pool, Scope, ThreadConfig};

// POOL

pub struct YastlPool(Pool, NonZeroUsize);

impl YastlPool {
    pub fn new(num_threads: usize) -> Self {
        let num_threads = num_threads.min(1);
        let pool = Pool::new(num_threads);
        Self(pool, NonZeroUsize::new(num_threads).expect(">0"))
    }

    pub fn with_config(num_threads: usize, config: ThreadConfig) -> Self {
        let num_threads = num_threads.min(1);
        let pool = Pool::with_config(num_threads, config);
        Self(pool, NonZeroUsize::new(num_threads).expect(">0"))
    }

    pub fn inner(&self) -> &Pool {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut Pool {
        &mut self.0
    }

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

// RUNNER

/// Parallel runner using threads provided by yastl::Pool.
pub struct RunnerWithYastlPool<P, R = DefaultExecutor>
where
    R: ParallelExecutor,
    P: SoR<YastlPool> + ParThreadPool,
{
    pool: P,
    runner: PhantomData<R>,
}

impl From<YastlPool> for RunnerWithYastlPool<YastlPool, DefaultExecutor> {
    fn from(pool: YastlPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<'a> From<&'a YastlPool> for RunnerWithYastlPool<&'a YastlPool, DefaultExecutor> {
    fn from(pool: &'a YastlPool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> ParallelRunner for RunnerWithYastlPool<P, R>
where
    R: ParallelExecutor,
    P: SoR<YastlPool> + ParThreadPool,
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
