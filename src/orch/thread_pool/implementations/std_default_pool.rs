use crate::{env::MAX_NUM_THREADS_ENV_VARIABLE, orch::ParThreadPool};
use std::num::NonZeroUsize;

const MAX_UNSET_NUM_THREADS: usize = 8;

pub struct StdDefaultPool {
    max_num_threads: NonZeroUsize,
}

impl Default for StdDefaultPool {
    fn default() -> Self {
        let env_max_num_threads = match std::env::var(MAX_NUM_THREADS_ENV_VARIABLE) {
            Ok(s) => match s.parse::<usize>() {
                Ok(0) => None,    // consistent with .num_threads(0) representing no bound
                Ok(x) => Some(x), // set to a positive bound
                Err(_e) => None,  // not a number, ignored assuming no bound
            },
            Err(_e) => None, // not set, no bound
        };

        let ava_max_num_threads: Option<usize> =
            std::thread::available_parallelism().map(|x| x.into()).ok();

        let max_num_threads = match (env_max_num_threads, ava_max_num_threads) {
            (Some(env), Some(ava)) => env.min(ava),
            (Some(env), None) => env,
            (None, Some(ava)) => ava,
            (None, None) => MAX_UNSET_NUM_THREADS,
        };

        let max_num_threads = NonZeroUsize::new(max_num_threads.max(1)).expect(">=1");

        Self { max_num_threads }
    }
}

impl ParThreadPool for StdDefaultPool {
    type ScopeRef<'s, 'env, 'scope>
        = &'s std::thread::Scope<'s, 'env>
    where
        'scope: 's,
        'env: 'scope + 's;

    fn max_num_threads(&self) -> NonZeroUsize {
        self.max_num_threads
    }

    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
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
