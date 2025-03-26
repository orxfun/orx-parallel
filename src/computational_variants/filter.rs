use super::map::ParMap;
use crate::{
    computations::map_self_atom,
    map_filter_map::{Atom, Mfm},
    runner::{DefaultRunner, ParallelRunner},
    ChunkSize, CollectOrdering, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParFilter<I, F, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync,
{
    iter: I,
    params: Params,
    filter: F,
    phantom: PhantomData<R>,
}

impl<I, F, R> ParFilter<I, F, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    F: Fn(&I::Item) -> bool + Send + Sync,
{
    pub(crate) fn new(params: Params, iter: I, filter: F) -> Self {
        Self {
            iter,
            params,
            filter,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, F) {
        (self.params, self.iter, self.filter)
    }

    fn mfm(
        self,
    ) -> Mfm<
        I,
        I::Item,
        Atom<I::Item>,
        I::Item,
        Atom<I::Item>,
        impl Fn(I::Item) -> Atom<I::Item>,
        impl Fn(&I::Item) -> bool,
        impl Fn(I::Item) -> Atom<I::Item>,
    > {
        let (params, iter, filter) = self.destruct();
        Mfm::new(params, iter, map_self_atom, filter, map_self_atom)
    }
}

// impl<I, F, R> ParIter<R> for ParFilter<I, F, R>
// where
//     R: ParallelRunner,
//     I: ConcurrentIter,
//     F: Fn(&I::Item) -> bool + Send + Sync,
// {
//     type Item = I::Item;

//     fn con_iter(&self) -> &impl ConcurrentIter {
//         &self.iter
//     }

//     // params transformations

//     fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
//         self.params = self.params.with_num_threads(num_threads);
//         self
//     }

//     fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
//         self.params = self.params.with_chunk_size(chunk_size);
//         self
//     }

//     fn collect_ordering(mut self, collect: CollectOrdering) -> Self {
//         self.params = self.params.with_collect_ordering(collect);
//         self
//     }

//     fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
//         ParFilter::new(self.params, self.iter, self.filter)
//     }

//     // computation transformations

//     fn map<Out, Map>(self, map: Map) -> impl ParIter<Item = Out>
//     where
//         Out: Send + Sync,
//         Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
//     {
//         let (params, iter, filter) = self.destruct();

//         todo!()
//     }

//     // collect

//     fn collect_into<C>(self, output: C) -> C
//     where
//         C: ParCollectInto<Self::Item>,
//     {
//         todo!()
//     }
// }
