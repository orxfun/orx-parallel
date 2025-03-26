use crate::{
    computations::{filter_true, map_self_atom},
    map_filter_map::{Atom, Mfm},
    runner::{DefaultRunner, ParallelRunner},
    ChunkSize, CollectOrdering, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParMap<I, O, M, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    map: M,
    phantom: PhantomData<R>,
}

impl<I, O, M, R> ParMap<I, O, M, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    pub(crate) fn new(params: Params, iter: I, map: M) -> Self {
        Self {
            iter,
            params,
            map,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M) {
        (self.params, self.iter, self.map)
    }

    fn mfm(
        self,
    ) -> Mfm<
        I,
        O,
        Atom<O>,
        O,
        Atom<O>,
        impl Fn(I::Item) -> Atom<O>,
        impl Fn(&O) -> bool,
        impl Fn(O) -> Atom<O>,
    > {
        let (params, iter, map) = self.destruct();
        let map1 = move |x| map_self_atom(map(x));
        Mfm::new(params, iter, map1, filter_true, map_self_atom)
    }
}

impl<I, O, M, R> ParIter<R> for ParMap<I, O, M, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    type Item = O;

    fn con_iter(&self) -> &impl ConcurrentIter {
        &self.iter
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    fn collect_ordering(mut self, collect: CollectOrdering) -> Self {
        self.params = self.params.with_collect_ordering(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        ParMap::new(self.params, self.iter, self.map)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (params, iter, map1) = self.destruct();
        let map = move |x| map(map1(x));
        ParMap::new(params, iter, map)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.collect_into::<R, _, _, _, _, _, _, _>(self.mfm(), true)
    }
}
