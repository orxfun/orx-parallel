use crate::ParThreadPool;
use core::num::NonZeroUsize;

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
