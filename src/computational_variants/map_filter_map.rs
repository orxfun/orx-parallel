use crate::{
    computations::{Mfm, Values},
    runner::{DefaultRunner, ParallelRunner},
    ChunkSize, CollectOrdering, NumThreads, ParCollectInto, ParIter, Params,
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParMapFilterMap<I, T, Vt, O, Vo, M1, F, M2, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    T: Send + Sync,
    Vt: Values<Item = T> + Send + Sync,
    O: Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
{
    mfm: Mfm<I, T, Vt, O, Vo, M1, F, M2>,
    phantom: PhantomData<R>,
}

impl<I, T, Vt, O, Vo, M1, F, M2, R> ParMapFilterMap<I, T, Vt, O, Vo, M1, F, M2, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    T: Send + Sync,
    Vt: Values<Item = T> + Send + Sync,
    O: Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
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

impl<I, T, Vt, O, Vo, M1, F, M2, R> ParIter<R> for ParMapFilterMap<I, T, Vt, O, Vo, M1, F, M2, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    T: Send + Sync,
    Vt: Values<Item = T> + Send + Sync,
    O: Send + Sync,
    Vo: Values<Item = O> + Send + Sync,
    M1: Fn(I::Item) -> Vt + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    M2: Fn(T) -> Vo + Send + Sync,
{
    type Item = O;

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
        let map23 = move |t: T| {
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
        output.mfm_collect_into::<R, _, _, _, _, _, _, _>(self.mfm)
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
