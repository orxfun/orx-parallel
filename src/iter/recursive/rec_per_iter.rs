use crate::{
    ParallelRunner,
    computational_variants::{Par, ParMap, ParXap},
    generic_values::{TransformableValues, runner_results::Infallible},
};
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter, implementations::ConIterVec};
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIterCore, Size};

type Rec<S, I, E> = ConcurrentRecursiveIterCore<S, <I as IntoIterator>::Item, E, I>;

impl<S, E, I, R> Par<Rec<S, I, E>, R>
where
    S: Size,
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Send,
    E: Fn(&I::Item) -> I + Sync,
    R: ParallelRunner,
{
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks.
    ///
    /// This might increase performance in certain cases; however, requires storing the flattened tasks.
    /// Therefore, it fits best to situations where the input elements are not very large.
    ///
    /// See [`into_par_rec`] and [`into_par_rec_exact`] for examples.
    ///
    /// [`into_par_rec`]: crate::IntoParIterRec::into_par_rec
    /// [`into_par_rec_exact`]: crate::IntoParIterRecExact::into_par_rec_exact
    pub fn into_eager(self) -> Par<ConIterVec<I::Item>, R> {
        let (orchestrator, params, iter) = self.destruct();
        let items = collect_items(iter);
        let iter = items.into_con_iter();
        Par::new(orchestrator, params, iter)
    }
}

impl<S, E, I, R, O, M1> ParMap<Rec<S, I, E>, O, M1, R>
where
    S: Size,
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Send,
    E: Fn(&I::Item) -> I + Sync,
    R: ParallelRunner,
    M1: Fn(I::Item) -> O + Sync,
{
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks.
    ///
    /// This might increase performance in certain cases; however, requires storing the flattened tasks.
    /// Therefore, it fits best to situations where the input elements are not very large.
    ///
    /// See [`into_par_rec`] and [`into_par_rec_exact`] for examples.
    ///
    /// [`into_par_rec`]: crate::IntoParIterRec::into_par_rec
    /// [`into_par_rec_exact`]: crate::IntoParIterRecExact::into_par_rec_exact
    pub fn into_eager(self) -> ParMap<ConIterVec<I::Item>, O, M1, R> {
        let (orchestrator, params, iter, map1) = self.destruct();
        let items = collect_items(iter);
        let iter = items.into_con_iter();
        ParMap::new(orchestrator, params, iter, map1)
    }
}

impl<S, E, I, R, Vo, X1> ParXap<Rec<S, I, E>, Vo, X1, R>
where
    S: Size,
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Send,
    E: Fn(&I::Item) -> I + Sync,
    R: ParallelRunner,
    X1: Fn(I::Item) -> Vo + Sync,
    Vo: TransformableValues<Fallibility = Infallible>,
{
    /// Even with exact length, a recursive parallel iterator is much more dynamic than a flat parallel
    /// iterator. This dynamic nature of shrinking and growing concurrently requires a greater parallelization
    /// overhead. An alternative approach is to eagerly discover all tasks and then perform the parallel
    /// computation over the flattened input of tasks.
    ///
    /// This might increase performance in certain cases; however, requires storing the flattened tasks.
    /// Therefore, it fits best to situations where the input elements are not very large.
    ///
    /// See [`into_par_rec`] and [`into_par_rec_exact`] for examples.
    ///
    /// [`into_par_rec`]: crate::IntoParIterRec::into_par_rec
    /// [`into_par_rec_exact`]: crate::IntoParIterRecExact::into_par_rec_exact
    pub fn into_eager(self) -> ParXap<ConIterVec<I::Item>, Vo, X1, R> {
        let (orchestrator, params, iter, xap1) = self.destruct();
        let items = collect_items(iter);
        let iter = items.into_con_iter();
        ParXap::new(orchestrator, params, iter, xap1)
    }
}

fn collect_items<S, I, E>(iter: Rec<S, I, E>) -> Vec<I::Item>
where
    S: Size,
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Send,
    E: Fn(&I::Item) -> I + Sync,
{
    match iter.try_get_len() {
        Some(len) => {
            let mut items = Vec::with_capacity(len);
            items.extend(iter.into_seq_iter());
            items
        }
        None => iter.into_seq_iter().collect(),
    }
}
