use std::num::NonZeroUsize;

use super::par_scope::ParScope;
use crate::orch::num_spawned::NumSpawned;
use orx_concurrent_bag::ConcurrentBag;

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

    fn max_num_threads(&self) -> NonZeroUsize;

    // derived

    fn run<S, F>(&mut self, do_spawn: S, thread_do: F) -> NumSpawned
    where
        S: Fn(NumSpawned) -> bool + Sync,
        F: Fn() + Sync,
    {
        let mut nt = NumSpawned::zero();
        self.scope(|s| {
            while do_spawn(nt) {
                nt.increment();
                Self::run_in_scope(&s, &thread_do);
            }
        });
        nt
    }

    fn map<S, M, T, E>(
        &mut self,
        do_spawn: S,
        thread_map: M,
        max_num_threads: NonZeroUsize,
    ) -> (NumSpawned, Result<Vec<T>, E>)
    where
        S: Fn(NumSpawned) -> bool + Sync,
        M: Fn() -> Result<T, E> + Sync,
        T: Send,
        E: Send,
    {
        let mut nt = NumSpawned::zero();
        let thread_results = ConcurrentBag::with_fixed_capacity(max_num_threads.into());
        let work = || _ = thread_results.push(thread_map());
        self.scope(|s| {
            while do_spawn(nt) {
                nt.increment();
                Self::run_in_scope(&s, &work);
            }
        });

        let mut results = vec![];
        for r in thread_results.into_inner() {
            match r {
                Ok(x) => results.push(x),
                Err(e) => return (nt, Err(e)),
            }
        }

        (nt, Ok(results))
    }
}
