use crate::{
    ParallelRunner,
    computational_variants::{Par, ParMap, ParXap},
    generic_values::{TransformableValues, runner_results::Infallible},
};
use orx_concurrent_iter::{ConcurrentIter, IntoConcurrentIter, implementations::ConIterVec};
use orx_concurrent_recursive_iter::ConcurrentRecursiveIter;

type Rec<I, E> = ConcurrentRecursiveIter<<I as IntoIterator>::Item, E, I>;

impl<E, I, R> Par<Rec<I, E>, R>
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Send,
    E: Fn(&I::Item) -> I + Sync,
    R: ParallelRunner,
{
    pub fn into_eager(self) -> Par<ConIterVec<I::Item>, R> {
        let (orchestrator, params, iter) = self.destruct();
        let items: Vec<_> = iter.into_seq_iter().collect();
        let iter = items.into_con_iter();
        Par::new(orchestrator, params, iter)
    }
}

impl<E, I, R, O, M1> ParMap<Rec<I, E>, O, M1, R>
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Send,
    E: Fn(&I::Item) -> I + Sync,
    R: ParallelRunner,
    M1: Fn(I::Item) -> O + Sync,
{
    pub fn into_eager(self) -> ParMap<ConIterVec<I::Item>, O, M1, R> {
        let (orchestrator, params, iter, map1) = self.destruct();
        let items: Vec<_> = iter.into_seq_iter().collect();
        let iter = items.into_con_iter();
        ParMap::new(orchestrator, params, iter, map1)
    }
}

impl<E, I, R, Vo, X1> ParXap<Rec<I, E>, Vo, X1, R>
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    I::Item: Send,
    E: Fn(&I::Item) -> I + Sync,
    R: ParallelRunner,
    X1: Fn(I::Item) -> Vo + Sync,
    Vo: TransformableValues<Fallibility = Infallible>,
{
    pub fn into_eager(self) -> ParXap<ConIterVec<I::Item>, Vo, X1, R> {
        let (orchestrator, params, iter, xap1) = self.destruct();
        let items: Vec<_> = iter.into_seq_iter().collect();
        let iter = items.into_con_iter();
        ParXap::new(orchestrator, params, iter, xap1)
    }
}
