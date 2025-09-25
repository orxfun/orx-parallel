use crate::ParThreadPool;
use core::num::NonZeroUsize;

/// A 'thread pool' with [`max_num_threads`] of 1.
/// All computations using this thread pool are executed sequentially by the main thread.
///
/// This is the default thread pool used when "std" feature is disabled.
/// Note that the thread pool to be used for a parallel computation can be set by the
/// [`with_runner`] transformation separately for each parallel iterator.
///
/// [`max_num_threads`]: ParThreadPool::max_num_threads
/// [`with_runner`]: crate::ParIter::with_runner
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
