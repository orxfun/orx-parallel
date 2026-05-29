use crate::parameters::{NumThreads, Params};
use core::num::NonZeroUsize;

pub trait ParThreadPool {
    /// Scope type of the thread pool.
    type ScopeRef<'s, 'env, 'scope>
    where
        'scope: 's,
        'env: 'scope + 's;

    /// Executes the `work` within scope `s`.
    fn run_in_scope<'s, 'env, 'scope, W>(s: &Self::ScopeRef<'s, 'env, 'scope>, work: W)
    where
        'scope: 's,
        'env: 'scope + 's,
        W: Fn() + Send + 'scope + 'env;

    /// Executes the scoped computation `f`.
    fn scoped_computation<'env, 'scope, F>(&'env mut self, f: F)
    where
        'env: 'scope,
        for<'s> F: FnOnce(Self::ScopeRef<'s, 'env, 'scope>) + Send;

    /// Returns the maximum number of threads available in the pool.
    fn max_num_threads(&self) -> NonZeroUsize;

    // provided

    /// Returns the maximum number of threads that can be used for the computation defined by
    /// the `params` and input `iter_len`.
    fn max_num_threads_for_computation(
        &self,
        params: Params,
        size_hint: (usize, Option<usize>),
    ) -> usize {
        let ava = self.max_num_threads();

        let req = match (size_hint.1, params.num_threads) {
            (Some(len_ub), NumThreads::Auto) => NonZeroUsize::new(len_ub.max(1)).expect(">0"),
            (Some(len_ub), NumThreads::Max(nt)) => {
                NonZeroUsize::new(len_ub.max(1)).expect(">0").min(nt)
            }
            (None, NumThreads::Auto) => NonZeroUsize::MAX,
            (None, NumThreads::Max(nt)) => nt,
        };

        core::cmp::min(req, ava).into()
    }
}
