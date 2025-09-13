use crate::orch::ParThreadPool;

#[derive(Default)]
pub struct StdDefaultPool;

impl ParThreadPool for StdDefaultPool {
    type ScopeZzz<'scope, 'env>
        = std::thread::Scope<'scope, 'env>
    where
        'env: 'scope;

    fn scope_zzz<'env, F, T>(&'env self, f: F) -> T
    where
        F: for<'scope> FnOnce(&'scope std::thread::Scope<'scope, 'env>) -> T,
    {
        std::thread::scope(f)
    }

    type ScopeRef<'s, 'env, 'scope>
        = &'s std::thread::Scope<'s, 'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn scope<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(&'s std::thread::Scope<'s, 'env>) + Send,
    {
        std::thread::scope(|s| f(&s))
    }

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: &'env W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Sync + 'scope + 'env,
    {
        s.spawn(|| work());
    }
}
