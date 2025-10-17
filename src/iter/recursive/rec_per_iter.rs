use crate::{
    ParallelRunner,
    computational_variants::{Par, ParMap},
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

// pub struct ParMap<I, O, M1, R = DefaultRunner>

// impl<E,I,R, O,M1> ParMap<>
