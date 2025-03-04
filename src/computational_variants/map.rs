use crate::{
    into_par_iter::IntoParIter, runner::DefaultRunner, ChunkSize, NumThreads, ParCollectInto,
    ParIter, Params,
};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    map: M,
}

impl<I, O, M> ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    pub(crate) fn new(params: Params, iter: I, map: M) -> Self {
        Self { iter, params, map }
    }

    fn destruct(self) -> (Params, I, M) {
        (self.params, self.iter, self.map)
    }
}

impl<I, O, M> IntoParIter for ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    type Item = O;

    fn into_par(self) -> impl ParIter<Item = Self::Item> {
        self
    }
}

impl<I, O, M> ParIter for ParMap<I, O, M>
where
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
        let (params, iter, map) = self.destruct();
        C::map_into::<_, _, DefaultRunner>(output, params, iter, map)
    }
}
