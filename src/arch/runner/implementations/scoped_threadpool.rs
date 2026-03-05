use crate::par_thread_pool::ParThreadPool;
use core::num::NonZeroUsize;
use scoped_threadpool::Pool;

impl ParThreadPool for Pool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s scoped_threadpool::Scope<'env, 'scope>
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
        for<'s> F: FnOnce(&'s scoped_threadpool::Scope<'env, 'scope>) + Send,
    {
        self.scoped(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new((self.thread_count() as usize).max(1)).expect(">0")
    }
}

impl ParThreadPool for &mut Pool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s scoped_threadpool::Scope<'env, 'scope>
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
        for<'s> F: FnOnce(&'s scoped_threadpool::Scope<'env, 'scope>) + Send,
    {
        self.scoped(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        NonZeroUsize::new((self.thread_count() as usize).max(1)).expect(">0")
    }
}
