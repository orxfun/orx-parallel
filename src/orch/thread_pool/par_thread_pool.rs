use super::par_scope::ParScope;
use crate::orch::num_spawned::NumSpawned;

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

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: &'env W)
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

    // derived

    fn run<S, W>(&mut self, do_spawn: S, work: W) -> NumSpawned
    where
        S: Fn(NumSpawned) -> bool + Sync,
        W: Fn() + Sync,
    {
        let mut nt = NumSpawned::zero();
        self.scope(|s| {
            while do_spawn(nt) {
                nt.increment();
                Self::run_in_scope(&s, &work);
            }
        });
        nt
    }
}
