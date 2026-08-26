use crate::common_par_traits::ParInfCommon;
use crate::infallible::Xap;
use crate::infallible::recursive::execution;
use crate::infallible::recursive::par::ParRec;
use crate::infallible::recursive::par_core::ParRecCore;
use crate::infallible::xap::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::pool::ParThreadPool;
use crate::runner::{DefaultRunner, ParRunner};
use crate::{Par, ParCollectInto};
use orx_concurrent_iter::ConcurrentIter;

/// Parallel iterator.
pub struct ParIter<I, X, Ix, Ex, R = DefaultRunner>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    Ix: IntoIterator<Item = X::I>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    iter: I,
    xap: X,
    exe: R,
    params: Params,
    extend: Ex,
}

impl<I, X, Ix, Ex, R> ParIter<I, X, Ix, Ex, R>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    Ix: IntoIterator<Item = X::I>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    pub(crate) fn new(iter: I, xap: X, exe: R, params: Params, extend: Ex) -> Self {
        Self {
            iter,
            xap,
            exe,
            params,
            extend,
        }
    }

    pub(super) fn with_xap<Y: Xap<I = I::Item>>(self, xap: Y) -> ParIter<I, Y, Ix, Ex, R> {
        ParIter::new(self.iter, xap, self.exe, self.params, self.extend)
    }

    fn destruct_x(self) -> (I, X, R, Params, Ex) {
        (self.iter, self.xap, self.exe, self.params, self.extend)
    }
}

impl<I, X, Ix, Ex, R> ParRecCore for ParIter<I, X, Ix, Ex, R>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    Ix: IntoIterator<Item = X::I>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    type Item = X::O;

    type Runner = R;

    type Input = I;

    type Xap = X;

    fn destruct(self) -> (Self::Input, Self::Xap, Self::Runner, Params) {
        (self.iter, self.xap, self.exe, self.params)
    }
}

impl<I, X, Ix, Ex, R> ParInfCommon for ParIter<I, X, Ix, Ex, R>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    Ix: IntoIterator<Item = X::I>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    type CommonItem = X::O;

    fn common_collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::CommonItem>,
        Self::CommonItem: Send,
    {
        self.collect_into(dst);
    }
}

impl<I, X, Ix, Ex, R> ParRec for ParIter<I, X, Ix, Ex, R>
where
    I: IntoIterator,
    X: Xap<I = I::Item>,
    R: ParRunner,
    Ix: IntoIterator<Item = X::I>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParRec<Item = Self::Item, Xap = Self::Xap, Input = Self::Input> {
        let (iter, xap, _, params, extend) = self.destruct_x();
        ParIter::new(iter, xap, runner, params, extend)
    }

    fn runner_with_diagnostics(
        self,
    ) -> impl ParRec<Item = Self::Item, Xap = Self::Xap, Input = Self::Input> {
        let (iter, xap, exe, params, extend) = self.destruct_x();
        ParIter::new(iter, xap, exe.with_diagnostics(), params, extend)
    }

    fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.params = self.params.with_collect_ordering(collect);
        self
    }

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParRec<Item = Q, Xap = MapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(Self::Item) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParRec<Item = Self::Item, Xap = InsOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParRec<Item = Self::Item, Xap = FilOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParRec<Item = Q, Xap = FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParRec<Item = V::Item, Xap = FlatMapOf<Self::Xap, V, H>, Input = Self::Input>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        self.with_xap(xap)
    }

    fn flatten(
        self,
    ) -> impl ParRec<
        Item = <Self::Item as IntoIterator>::Item,
        Xap = FlattenOf<Self::Xap>,
        Input = Self::Input,
    >
    where
        Self::Item: IntoIterator,
    {
        let xap = self.xap.flatten();
        self.with_xap(xap)
    }

    // compute

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync,
    {
        let (iter, x, exe, params, extend) = self.destruct_x();
        execution::next_any(exe, params, iter, x, extend);

        // match params.iteration_order {
        //     IterationOrder::Ordered => exe.next(params, iter, x).map(|x| x.val),
        //     IterationOrder::Arbitrary => exe.next_any(params, iter, x),
        // }
        todo!()
    }

    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
    {
        todo!()
    }

    fn collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
    {
        todo!()
    }

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
    {
        todo!()
    }
}
