use crate::{
    computations::{Mfm, Values},
    runner::{DefaultRunner, ParallelRunner},
    ChunkSize, CollectOrdering, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParMapFilterMap<I, Vt, Vo, M1, F, M2, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vt::Item: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    mfm: Mfm<I, Vt, Vo, M1, F, M2>,
    phantom: PhantomData<R>,
}

impl<I, Vt, Vo, M1, F, M2, R> ParMapFilterMap<I, Vt, Vo, M1, F, M2, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vt::Item: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    pub(crate) fn new(params: Params, iter: I, map1: M1, filter: F, map2: M2) -> Self {
        Self {
            mfm: Mfm::new(params, iter, map1, filter, map2),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M1, F, M2) {
        self.mfm.destruct()
    }
}

impl<I, Vt, Vo, M1, F, M2, R> ParIter<R> for ParMapFilterMap<I, Vt, Vo, M1, F, M2, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vt: Values + Send + Sync,
    Vt::Item: Send + Sync,
    Vo: Values + Send + Sync,
    Vo::Item: Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&Vt::Item) -> bool + Send + Sync,
    M2: Fn(Vt::Item) -> Vo + Send + Sync,
{
    type Item = Vo::Item;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.mfm.iter()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.mfm.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.mfm.chunk_size(chunk_size);
        self
    }

    fn collect_ordering(mut self, collect: CollectOrdering) -> Self {
        self.mfm.collect_ordering(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        let (params, iter, map1, filter, map2) = self.destruct();
        ParMapFilterMap::new(params, iter, map1, filter, map2)
    }

    // computation transformations

    fn map<Out, Map>(self, map3: Map) -> impl ParIter<R, Item = Out>
    where
        Out: Send + Sync,
        Map: Fn(Self::Item) -> Out + Send + Sync + Clone,
    {
        let (params, iter, map1, filter, map2) = self.destruct();
        let map23 = move |t: Vt::Item| {
            let vo = map2(t);
            vo.map(map3.clone())
        };

        ParMapFilterMap::new(params, iter, map1, filter, map23)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Send + Sync,
    {
        todo!();
        self
    }

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.mfm_collect_into::<R, _, _, _, _, _, _>(self.mfm)
    }
}
