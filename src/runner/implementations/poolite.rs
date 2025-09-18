use crate::{DefaultExecutor, ParThreadPool, ParallelExecutor, runner::ParallelRunner};
use core::{marker::PhantomData, num::NonZeroUsize};
use orx_self_or::SoR;
use poolite::{Pool, Scoped};

// POOL

impl ParThreadPool for Pool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s Scoped<'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.push(work);
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s Scoped<'env, 'scope>) + Send,
    {
        self.scoped(f);
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.threads_future().max(1)).expect(">0")
    }
}

impl ParThreadPool for &Pool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s Scoped<'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        s.push(work);
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s Scoped<'env, 'scope>) + Send,
    {
        self.scoped(f);
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.threads_future().max(1)).expect(">0")
    }
}

// RUNNER

/// Parallel runner using threads provided by poolite::Pool.
pub struct RunnerWithPoolitePool<P, R = DefaultExecutor>
where
    R: ParallelExecutor,
    P: SoR<Pool> + ParThreadPool,
{
    pool: P,
    runner: PhantomData<R>,
}

impl From<Pool> for RunnerWithPoolitePool<Pool, DefaultExecutor> {
    fn from(pool: Pool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<'a> From<&'a Pool> for RunnerWithPoolitePool<&'a Pool, DefaultExecutor> {
    fn from(pool: &'a Pool) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> ParallelRunner for RunnerWithPoolitePool<P, R>
where
    R: ParallelExecutor,
    P: SoR<Pool> + ParThreadPool,
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
