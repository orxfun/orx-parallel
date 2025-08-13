use crate::ParIterResultNew;
use crate::par_iter_result::ParIterResultStruct;
use crate::values::{Values, WhilstOk};
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParIter, ParIterUsing, Params,
    computations::X,
    runner::{DefaultRunner, ParallelRunner},
    using::{UsingClone, UsingFun, computational_variants::UParXap},
};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator that xaps inputs.
///
/// *xap* is a generalization of  one-to-one map, filter-map and flat-map operations.
pub struct ParXap<I, Vo, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Error: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
    x: X<I, Vo, M1>,
    phantom: PhantomData<R>,
}

impl<I, Vo, M1, R> ParXap<I, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Error: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
    pub(crate) fn new(params: Params, iter: I, x1: M1) -> Self {
        Self {
            x: X::new(params, iter, x1),
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, M1) {
        self.x.destruct()
    }
}

unsafe impl<I, Vo, M1, R> Send for ParXap<I, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Error: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
}

unsafe impl<I, Vo, M1, R> Sync for ParXap<I, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Error: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
}

impl<I, Vo, M1, R> ParIter<R> for ParXap<I, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Error: Send,
    M1: Fn(I::Item) -> Vo + Sync,
{
    type Item = Vo::Item;

    type ConIter = I;

    fn con_iter(&self) -> &impl ConcurrentIter {
        self.x.iter()
    }

    fn params(&self) -> Params {
        self.x.params()
    }

    // params transformations

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.x.num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.x.chunk_size(chunk_size);
        self
    }

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.x.iteration_order(collect);
        self
    }

    fn with_runner<Q: ParallelRunner>(self) -> impl ParIter<Q, Item = Self::Item> {
        let (params, iter, map1) = self.destruct();
        ParXap::new(params, iter, map1)
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
        let (params, iter, x1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| x1(t);
        UParXap::new(using, params, iter, m1)
    }

    fn using_clone<U>(
        self,
        using: U,
    ) -> impl ParIterUsing<UsingClone<U>, R, Item = <Self as ParIter<R>>::Item>
    where
        U: Clone + Send,
    {
        let using = UsingClone::new(using);
        let (params, iter, x1) = self.destruct();
        let m1 = move |_: &mut U, t: I::Item| x1(t);
        UParXap::new(using, params, iter, m1)
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIter<R, Item = Out>
    where
        Map: Fn(Self::Item) -> Out + Sync + Clone,
    {
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.map(map.clone())
        };

        ParXap::new(params, iter, x1)
    }

    fn filter<Filter>(self, filter: Filter) -> impl ParIter<R, Item = Self::Item>
    where
        Filter: Fn(&Self::Item) -> bool + Sync + Clone,
    {
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let values = x1(i);
            values.filter(filter.clone())
        };
        ParXap::new(params, iter, x1)
    }

    fn flat_map<IOut, FlatMap>(self, flat_map: FlatMap) -> impl ParIter<R, Item = IOut::Item>
    where
        IOut: IntoIterator,
        FlatMap: Fn(Self::Item) -> IOut + Sync + Clone,
    {
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.flat_map(flat_map.clone())
        };
        ParXap::new(params, iter, x1)
    }

    fn filter_map<Out, FilterMap>(self, filter_map: FilterMap) -> impl ParIter<R, Item = Out>
    where
        FilterMap: Fn(Self::Item) -> Option<Out> + Sync + Clone,
    {
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.filter_map(filter_map.clone())
        };
        ParXap::new(params, iter, x1)
    }

    fn take_while<While>(self, take_while: While) -> impl ParIter<R, Item = Self::Item>
    where
        While: Fn(&Self::Item) -> bool + Sync + Clone,
    {
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            let vo = x1(i);
            vo.whilst(take_while.clone())
        };
        ParXap::new(params, iter, x1)
    }

    fn map_while_ok<Out, Err, MapWhileOk>(
        self,
        map_while_ok: MapWhileOk,
    ) -> ParIterResultStruct<
        Self::ConIter,
        Out,
        Err,
        impl Fn(<Self::ConIter as ConcurrentIter>::Item) -> WhilstOk<Out, Err> + Sync,
        R,
    >
    where
        MapWhileOk: Fn(Self::Item) -> Result<Out, Err> + Sync + Clone,
        Err: Send + Sync,
    {
        // TODO: this is wrong! we need vector of results
        let con_iter_len = self.con_iter().try_get_len();
        let (params, iter, x1) = self.destruct();
        let x1 = move |i: I::Item| {
            WhilstOk::<Out, Err>::new(map_while_ok(x1(i).values().into_iter().next().unwrap()))
        };
        let x = X::new(params, iter, x1);
        ParIterResultStruct::<I, Out, Err, _, R>::new(x, con_iter_len)
    }

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        output.x_collect_into::<R, _, _, _>(self.x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Option<Self::Item>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.x.reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
    {
        match self.params().iteration_order {
            IterationOrder::Ordered => self.x.next::<R>().1,
            IterationOrder::Arbitrary => self.x.next_any::<R>().1,
        }
    }
}

// impl<I, Vo, M1, R> ParIterResultNew for ParXap<I, Vo, M1, R>
// where
//     R: ParallelRunner,
//     I: ConcurrentIter,
//     Vo: Values,
//     Vo::Error: Send,
//     M1: Fn(I::Item) -> Vo + Sync,
// {
// }
