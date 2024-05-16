use crate::{ops, params, ParMap};
use orx_concurrent_iter::{ConcurrentIter, ExactSizeConcurrentIter};
use std::num::NonZeroUsize;

pub struct Par<Data>
where
    Data: ConcurrentIter,
{
    data: Data,
    num_threads: Option<NonZeroUsize>,
    chunk_size: Option<NonZeroUsize>,
}

impl<Data> Par<Data>
where
    Data: ConcurrentIter,
{
    pub(crate) fn from_data(data: Data) -> Self {
        Self {
            data,
            num_threads: None,
            chunk_size: None,
        }
    }

    pub(crate) fn data(&self) -> &Data {
        &self.data
    }

    pub(crate) fn into_data(self) -> Data {
        self.data
    }

    pub(crate) fn map_data<NewData: ConcurrentIter>(
        self,
        map: impl Fn(Data) -> NewData,
    ) -> Par<NewData> {
        let data = map(self.data);
        Par {
            data,
            num_threads: self.num_threads,
            chunk_size: self.chunk_size,
        }
    }

    pub(crate) fn eval_num_threads_chunk_size(&self) -> (usize, usize) {
        let num_threads = params::num_threads(self.num_threads.map(|x| x.into()));
        let chunk_size = params::chunk_size(
            self.chunk_size.map(|x| x.into()),
            self.data.try_get_len(),
            num_threads,
        );
        (num_threads, chunk_size)
    }
}

impl<Data> Par<Data>
where
    Data: ConcurrentIter + ExactSizeConcurrentIter,
{
    pub(crate) fn exact_len(&self) -> usize {
        self.data.len()
    }
}

// API

impl<Data> Par<Data>
where
    Data: ConcurrentIter,
{
    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads =
            Some(NonZeroUsize::new(num_threads).expect("Number of threads must be positive"));
        self
    }

    pub fn chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = Some(NonZeroUsize::new(chunk_size).expect("Chunk size must be positive"));
        self
    }

    #[inline]
    pub fn map<Out, Map>(self, map: Map) -> ParMap<Data, Out, Map>
    where
        Map: Fn(Data::Item) -> Out + Send + Sync,
        Out: Send + Sync,
    {
        ParMap::new(self, map)
    }

    #[inline]
    pub fn reduce<Reduce>(self, reduce: Reduce) -> Option<Data::Item>
    where
        Data: ConcurrentIter,
        Data::Item: Send,
        Reduce: Fn(Data::Item, Data::Item) -> Data::Item + Send + Sync,
    {
        ops::reduce::reduce(self, reduce)
    }

    #[inline]
    pub fn map_reduce<Out, Map, Reduce>(self, map: Map, reduce: Reduce) -> Option<Out>
    where
        Out: Send,
        Map: Fn(Data::Item) -> Out + Send + Sync,
        Reduce: Fn(Out, Out) -> Out + Send + Sync,
    {
        ops::map_reduce::map_reduce(self, map, reduce)
    }
}
