use crate::orch::ParThreadPool;
use std::num::NonZeroUsize;

impl ParThreadPool for rayon::ThreadPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s rayon::Scope<'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: &'env W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Sync + 'scope + 'env,
    {
        s.spawn(|_| work());
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s rayon::Scope<'scope>) + Send,
    {
        self.scope(f)
    }

    fn max_num_threads(&self) -> NonZeroUsize {
        match self.current_num_threads() {
            0 => NonZeroUsize::new(1).expect(">0"),
            n => NonZeroUsize::new(n).expect(">0"),
        }
    }
}
