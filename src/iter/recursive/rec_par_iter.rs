use crate::{
    ParIter, ParallelRunner,
    computational_variants::{Par, ParMap, ParXap},
    generic_values::{TransformableValues, runner_results::Infallible},
};
use orx_concurrent_iter::{IntoConcurrentIter, implementations::ConIterVec};
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

type Rec<T, E> = ConcurrentRecursiveIter<T, E>;

impl<E, T, R> Par<Rec<T, E>, R>
where
    T: Send + Sync,
    E: Fn(&T, &Queue<T>) + Sync,
    R: ParallelRunner + Clone,
{
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks.
    ///
    /// The `linearize` approach works in two parallelization phases:
    /// * first phase to linearize the inputs in parallel over the non-linear data, and
    /// * second phase to perform the computation in parallel over the linear data.
    ///
    /// See [`into_par_rec`] and [`into_par_rec_exact`] for examples.
    ///
    /// [`into_par_rec`]: crate::IntoParIterRec::into_par_rec
    /// [`into_par_rec_exact`]: crate::IntoParIterRec::into_par_rec_exact
    pub fn linearize(self) -> Par<ConIterVec<T>, R> {
        let params = self.params();
        let orchestrator = self.orchestrator().clone();
        let items: Vec<_> = self.collect();
        let iter = items.into_con_iter();
        Par::new(orchestrator, params, iter)
    }
}

impl<E, T, R, O, M1> ParMap<Rec<T, E>, O, M1, R>
where
    T: Send + Sync,
    E: Fn(&T, &Queue<T>) + Sync,
    R: ParallelRunner + Clone,
    M1: Fn(T) -> O + Sync,
{
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks.
    ///
    /// The `linearize` approach works in two parallelization phases:
    /// * first phase to linearize the inputs in parallel over the non-linear data, and
    /// * second phase to perform the computation in parallel over the linear data.
    ///
    /// See [`into_par_rec`] and [`into_par_rec_exact`] for examples.
    ///
    /// [`into_par_rec`]: crate::IntoParIterRec::into_par_rec
    /// [`into_par_rec_exact`]: crate::IntoParIterRec::into_par_rec_exact
    pub fn linearize(self) -> ParMap<ConIterVec<T>, O, M1, R> {
        let (orchestrator, params, iter, map1) = self.destruct();
        let par = Par::new(orchestrator.clone(), params, iter);
        let items: Vec<_> = par.collect();
        let iter = items.into_con_iter();
        ParMap::new(orchestrator, params, iter, map1)
    }
}

impl<E, T, R, Vo, X1> ParXap<Rec<T, E>, Vo, X1, R>
where
    T: Send + Sync,
    E: Fn(&T, &Queue<T>) + Sync,
    R: ParallelRunner + Clone,
    X1: Fn(T) -> Vo + Sync,
    Vo: TransformableValues<Fallibility = Infallible>,
{
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks.
    ///
    /// The `linearize` approach works in two parallelization phases:
    /// * first phase to linearize the inputs in parallel over the non-linear data, and
    /// * second phase to perform the computation in parallel over the linear data.
    ///
    /// See [`into_par_rec`] and [`into_par_rec_exact`] for examples.
    ///
    /// [`into_par_rec`]: crate::IntoParIterRec::into_par_rec
    /// [`into_par_rec_exact`]: crate::IntoParIterRec::into_par_rec_exact
    pub fn linearize(self) -> ParXap<ConIterVec<T>, Vo, X1, R> {
        let (orchestrator, params, iter, xap1) = self.destruct();
        let par = Par::new(orchestrator.clone(), params, iter);
        let items: Vec<_> = par.collect();
        let iter = items.into_con_iter();
        ParXap::new(orchestrator, params, iter, xap1)
    }
}
