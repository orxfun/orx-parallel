use super::xap::ParXap;
use crate::ParIterResult;
use crate::computational_variants::result::ParMapResult;
use crate::computations::X;
use crate::par_iter_result::ParIterResult3;
use crate::values::{Vector, WhilstAtom, Okay};
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, ParIterUsing, Params,
    computations::M,
    runner::{DefaultRunner, ParallelRunner},
    using::{UsingClone, UsingFun, computational_variants::UParMap},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that maps inputs.
pub struct ParMap<I, O, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
    m: M<I, O, M1>,
    phantom: PhantomData<R>,
}

impl<I, O, M1, R> ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
    pub(crate) fn new(params: Params, iter: I, m1: M1) -> Self {
        Self {
            m: M::new(params, iter, m1),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M1) {
        self.m.destruct()
    }
}

unsafe impl<I, O, M1, R> Send for ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
}

unsafe impl<I, O, M1, R> Sync for ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
}

impl<I, O, M1, R> ParIter<R> for ParMap<I, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M1: Fn(I::Item) -> O + Sync,
{
    type Item = O;

    type ConIter = I;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.m.iter()
    }

    fn params(&self) -> Params {
        self.m.params()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.m.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.m.chunk_size(chunk_size);
        self
    }

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.m.iteration_order(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        let (params, iter, map) = self.destruct();
        ParMap::new(params, iter, map)
    }

    // using transformations

    fn using<U, F>(
        self,
        using: F,
    ) -> impl ParIterUsing<UsingFun<F, U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Send,
        F: FnMut(usize) -> U,
    {
        let using = UsingFun::new(using);
        let (params, iter, m1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| m1(t);
        UParMap::new(using, params, iter, m1)
    }

    fn using_clone<U>(
        self,
        using: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + Send,
    {
        let using = UsingClone::new(using);
        let (params, iter, m1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| m1(t);
        UParMap::new(using, params, iter, m1)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Map: Fn(Self::Item) -> Out + Sync,
    {
        let (params, iter, m1) = self.destruct();
        let m1 = move |x| map(m1(x));
        ParMap::new(params, iter, m1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Sync,
    {
        let (params, iter, m1) = self.destruct();

        let x1 = move |i: I::Item| {
            let value = m1(i);
            filter(&value).then_some(value)
        };
        ParXap::new(params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(Self::Item) -> IOut + Sync,
    {
        let (params, iter, m1) = self.destruct();
        let x1 = move |i: I::Item| Vector(flat_map(m1(i)));
        ParXap::new(params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync,
    {
        let (params, iter, m1) = self.destruct();
        let x1 = move |i: I::Item| filter_map(m1(i));
        ParXap::new(params, iter, x1)
    }

    fn take_while<While>(self, take_while: While) -> impl ParIter<R, Item = Self::Item>
    where
        While: Fn(&Self::Item) -> bool + Sync,
    {
        let (params, iter, m1) = self.destruct();
        let x1 = move |value: I::Item| WhilstAtom::new(m1(value), &take_while);
        ParXap::new(params, iter, x1)
    }

    fn map_while_ok<Out, Err, MapWhileOk>(
        self,
        map_while_ok: MapWhileOk,
    ) -> impl ParIterResult<R, Item = Out, Error = Err>
    where
        MapWhileOk: Fn(Self::Item) -> Result<Out, Err> + Sync + Clone,
    {
        let (params, iter, m1) = self.destruct();
        let map_res = move |value: I::Item| map_while_ok(m1(value));
        ParMapResult::new(iter, params, map_res)
    }

    // fn map_while_ok<Out, Err, MapWhileOk>(
    //     self,
    //     map_while_ok: MapWhileOk,
    // ) -> ParIterResult3<
    //     Self::ConIter,
    //     Out,
    //     Err,
    //     impl Fn(<Self::ConIter as ConcurrentIter>::Item) -> WhilstOk<Out, Err> + Sync,
    //     R,
    // >
    // where
    //     MapWhileOk: Fn(Self::Item) -> Result<Out, Err> + Sync + Clone,
    //     Err: Send + Sync,
    // {
    //     let con_iter_len = self.con_iter().try_get_len();
    //     let (params, iter, m1) = self.destruct();
    //     let x1 = move |i: I::Item| WhilstOk::<Out, Err>::new(map_while_ok(m1(i)));
    //     let x = X::new(params, iter, x1);
    //     ParIterResult3::<I, Out, Err, _, R>::new(x, con_iter_len)
    // }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.m_collect_into::<R, _, _>(self.m)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.m.reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        match self.params().iteration_order {
            IterationOrder::Ordered => self.m.next::<R>().1,
            IterationOrder::Arbitrary => self.m.next_any::<R>().1,
        }
    }
}
