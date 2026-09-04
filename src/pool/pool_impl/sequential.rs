use crate::pool::{ThreadPool, scope::Scope};
use core::num::NonZeroUsize;

/// A placeholder _thread pool_ allowed to use only the executing thread.
/// Therefore, all computations using this thread pool are executed sequentially.
///
/// This is the default **fallback** thread pool used when "std" feature is disabled.
/// Normally, in no-std environments thread pool to be used by the parallel computation
/// must be provided by the [`pool`] method to the parallel iterator.
///
/// Provided pool must implement [`ThreadPool`].
/// This crate provides optional or default implementations, which can be constructed
/// using the [`Pool`] helper type.
///
/// Note that the thread pool to be used for a parallel computation can be set by the
/// [`with_runner`] transformation separately for each parallel iterator.
///
/// [`pool`]: crate::Par::pool
/// [`ThreadPool`]: crate::ThreadPool
/// [`Pool`]: crate::Pool
#[derive(Default, Clone, Copy, Debug)]
pub struct SequentialPool;

pub struct SequentialScope;

impl<'s, 'env, 'scope> Scope<'s, 'env, 'scope> for SequentialScope {
    fn run<W>(&self, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env,
    {
        work();
    }
}

impl ThreadPool for SequentialPool {
    type ScopeRef<'s, 'env, 'scope>
        = SequentialScope
    where
        'scope: 's,
        'env: 'scope + 's;

    fn scope<'env, 'scope, F>(&'env self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(SequentialScope) + Send,
    {
        f(SequentialScope)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }
}

impl ThreadPool for &SequentialPool {
    type ScopeRef<'s, 'env, 'scope>
        = SequentialScope
    where
        'scope: 's,
        'env: 'scope + 's;

    fn scope<'env, 'scope, F>(&'env self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(SequentialScope) + Send,
    {
        f(SequentialScope)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }
}
