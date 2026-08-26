use crate::common_par_traits::ParInfCommon;
use crate::infallible::Xap;
use crate::infallible::par_core::ParCore;
use crate::infallible::par_runner::ParRunnerInfallible;
use crate::infallible::xap::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner};
use crate::{Par, ParCollectInto};
use orx_concurrent_iter::ConcurrentIter;

/// Parallel iterator.
pub struct ParIter<I, X, R = DefaultRunner>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    iter: I,
    xap: X,
    exe: R,
    params: Params,
}

impl<I, X, R> ParIter<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    pub(crate) fn new(iter: I, xap: X, exe: R, params: Params) -> Self {
        Self {
            iter,
            xap,
            exe,
            params,
        }
    }

    pub(super) fn with_xap<Y: Xap<I = I::Item>>(self, xap: Y) -> ParIter<I, Y, R> {
        ParIter::new(self.iter, xap, self.exe, self.params)
    }
}

impl<I, X, R> ParCore for ParIter<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    type Item = X::O;

    type Runner = R;

    type Input = I;

    type Xap = X;

    fn destruct(self) -> (Self::Input, Self::Xap, Self::Runner, Params) {
        (self.iter, self.xap, self.exe, self.params)
    }
}

impl<I, X, R> Par for ParIter<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl Par<Item = Self::Item, Xap = Self::Xap, Input = Self::Input> {
        let (iter, xap, _, params) = self.destruct();
        ParIter::new(iter, xap, runner, params)
    }

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl Par<Item = Self::Item, Xap = Self::Xap, Input = Self::Input> {
        let (iter, xap, exe, params) = self.destruct();
        ParIter::new(iter, xap, exe.with_diagnostics(), params)
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
    ) -> impl Par<Item = Q, Xap = MapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(X::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    fn inspect<H>(
        self,
        h: H,
    ) -> impl Par<Item = Self::Item, Xap = InsOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    fn filter<H>(
        self,
        h: H,
    ) -> impl Par<Item = Self::Item, Xap = FilOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl Par<Item = Q, Xap = FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl Par<Item = V::Item, Xap = FlatMapOf<Self::Xap, V, H>, Input = Self::Input>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        self.with_xap(xap)
    }

    fn flatten(
        self,
    ) -> impl Par<
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

    fn first(self) -> Option<X::O>
    where
        X::O: Send,
    {
        let (iter, x, mut exe, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(params, iter, x).map(|x| x.val),
            IterationOrder::Arbitrary => exe.next_any(params, iter, x),
        }
    }

    fn reduce<F>(self, f: F) -> Option<X::O>
    where
        F: Fn(X::O, X::O) -> X::O + Send + Copy,
        X::O: Send,
    {
        let (iter, x, mut exe, params) = self.destruct();
        exe.reduce(params, iter, x, f)
    }

    fn collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_col_into(dst, self),
            IterationOrder::Arbitrary => C::inf_arb_col_into(dst, self),
        }
    }

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        let mut dst = C::new_empty();
        self.collect_into(&mut dst);
        dst
    }
}

impl<I, X, R> ParInfCommon for ParIter<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item>,
    R: ParRunner,
{
    type CommonItem = <Self as ParCore>::Item;

    fn common_collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::CommonItem>,
        Self::CommonItem: Send,
    {
        self.collect_into(dst);
    }
}
