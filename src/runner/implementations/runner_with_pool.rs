use crate::{DefaultExecutor, ParThreadPool, ParallelExecutor, runner::ParallelRunner};
use core::marker::PhantomData;

pub struct RunnerWithPool<P, R = DefaultExecutor>
where
    P: ParThreadPool,
    R: ParallelExecutor,
{
    pool: P,
    runner: PhantomData<R>,
}

impl<P, R> Default for RunnerWithPool<P, R>
where
    P: ParThreadPool + Default,
    R: ParallelExecutor,
{
    fn default() -> Self {
        Self {
            pool: Default::default(),
            runner: PhantomData,
        }
    }
}

impl<P: ParThreadPool> From<P> for RunnerWithPool<P, DefaultExecutor> {
    fn from(pool: P) -> Self {
        Self {
            pool,
            runner: PhantomData,
        }
    }
}

impl<P, R> RunnerWithPool<P, R>
where
    P: ParThreadPool,
    R: ParallelExecutor,
{
    pub fn into_inner_pool(self) -> P {
        self.pool
    }
}

impl<P, R> ParallelRunner for RunnerWithPool<P, R>
where
    P: ParThreadPool,
    R: ParallelExecutor,
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
