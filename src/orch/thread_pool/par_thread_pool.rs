use super::par_scope::ParScope;

pub trait ParThreadPool {
    type ScopeZzz<'scope, 'env>: ParScope<'scope, 'env>
    where
        'env: 'scope;

    fn scope_zzz<'env, F, T>(&'env self, f: F) -> T
    where
        F: for<'scope> FnOnce(&'scope Self::ScopeZzz<'scope, 'env>) -> T;

    type ScopeRef<'s, 'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope2<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: &'env W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Sync + 'scope + 'env,
    {
    }

    fn scope<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(Self::ScopeRef<'s, 'env, 'scope>) + Send,
    {
        todo!()
    }
}
