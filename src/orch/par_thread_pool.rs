use crate::{generic_values::runner_results::Fallibility, orch::num_spawned::NumSpawned};
use alloc::vec::Vec;
use core::num::NonZeroUsize;
use orx_concurrent_bag::ConcurrentBag;

/// A thread pool that can be used for parallel computation.
///
/// # Examples
///
/// ```
/// ```
pub trait ParThreadPool {
    type ScopeRef<'s, 'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env;

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(Self::ScopeRef<'s, 'env, 'scope>) + Send;

    fn max_num_threads(&self) -> NonZeroUsize;
}

// derived

pub trait ParThreadPoolCompute: ParThreadPool {
    fn map_in_pool<F, S, M, T>(
        &mut self,
        do_spawn: S,
        thread_map: M,
        max_num_threads: NonZeroUsize,
    ) -> (NumSpawned, Result<Vec<T>, F::Error>)
    where
        F: Fallibility,
        S: Fn(NumSpawned) -> bool + Sync,
        M: Fn(NumSpawned) -> Result<T, F::Error> + Sync,
        T: Send,
        F::Error: Send,
    {
        let thread_map = &thread_map;
        let mut nt = NumSpawned::zero();
        let thread_results = ConcurrentBag::with_fixed_capacity(max_num_threads.into());
        let bag = &thread_results;
        self.scoped_computation(|s| {
            while do_spawn(nt) {
                let num_spawned = nt;
                nt.increment();
                let work = move || {
                    bag.push(thread_map(num_spawned));
                };
                Self::run_in_scope(&s, work);
            }
        });

        let thread_results: Vec<_> = thread_results.into_inner().into();
        let result = F::reduce_results(thread_results);

        (nt, result)
    }

    fn run_in_pool<S, F>(&mut self, do_spawn: S, thread_do: F) -> NumSpawned
    where
        S: Fn(NumSpawned) -> bool + Sync,
        F: Fn(NumSpawned) + Sync,
    {
        let thread_do = &thread_do;
        let mut nt = NumSpawned::zero();
        self.scoped_computation(|s| {
            while do_spawn(nt) {
                let num_spawned = nt;
                nt.increment();
                let work = move || thread_do(num_spawned);
                Self::run_in_scope(&s, work);
            }
        });
        nt
    }
}

impl<X: ParThreadPool> ParThreadPoolCompute for X {}

#[cfg(test)]
mod tsts {
    use crate::*;

    #[test]
    fn abc() {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();
    }
}
