use crate::{DefaultExecutor, ParThreadPool, runner::ParallelRunner};
use core::num::NonZeroUsize;

// POOL

#[derive(Default)]
pub struct SequentialPool;

impl ParThreadPool for SequentialPool {
    type ScopeRef<'s, 'env, 'scope>
        = ()
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(_: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        work()
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(()) + Send,
    {
        f(())
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new(1).expect(">0")
    }
}

// RUNNER

/// Sequential runner using using the main thread.
///
/// This is the default runner when "std" feature is not enabled.
///
/// Parallelization can be achieved by providing a parallel runner
/// using the [`with_runner`] method of parallel iterators.
///
/// [`with_runner`]: crate::ParIter::with_runner
#[derive(Default)]
pub struct SequentialRunner(SequentialPool);

impl ParallelRunner for SequentialRunner {
    type Executor = DefaultExecutor;

    type ThreadPool = SequentialPool;

    fn thread_pool(&self) -> &Self::ThreadPool {
        &self.0
    }

    fn thread_pool_mut(&mut self) -> &mut Self::ThreadPool {
        &mut self.0
    }
}
