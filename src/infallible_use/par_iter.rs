use crate::common_par_traits::ParInfCommon;
use crate::infallible_use::par::ParUse;
use crate::infallible_use::par_runner::ParRunnerInfallibleUse;
use crate::infallible_use::xap::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf};
use crate::infallible_use::{ParUseCore, XapUse};
use crate::parameters::{IterationOrder, Params};
use crate::runner::{DefaultRunner, ParRunner};
use crate::use_var::Use;
use crate::{ChunkSize, NumThreads, ParCollectInto};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParUseIter<U, I, X, R = DefaultRunner>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    using: U,
    iter: I,
    xap: X,
    exe: R,
    params: Params,
}

impl<U, I, X, R> ParUseIter<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    pub(crate) fn new(using: U, iter: I, xap: X, exe: R, params: Params) -> Self {
        Self {
            using,
            iter,
            xap,
            exe,
            params,
        }
    }

    pub(super) fn with_xap<Y: XapUse<U = U::Item, I = I::Item>>(
        self,
        xap: Y,
    ) -> ParUseIter<U, I, Y, R> {
        ParUseIter::new(self.using, self.iter, xap, self.exe, self.params)
    }
}

impl<U, I, X, R> ParUseCore for ParUseIter<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    type Item = X::O;

    type Runner = R;

    type Using = U;

    type Use = U::Item;

    type Input = I;

    type Xap = X;

    fn destruct(self) -> (Self::Using, Self::Input, Self::Xap, Self::Runner, Params) {
        (self.using, self.iter, self.xap, self.exe, self.params)
    }
}

impl<U, I, X, R> ParUse for ParUseIter<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = Self::Xap, Input = Self::Input> {
        let (using, iter, xap, _, params) = self.destruct();
        ParUseIter {
            using,
            iter,
            xap,
            exe: runner,
            params,
        }
    }

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = Self::Xap, Input = Self::Input> {
        let (using, iter, xap, exe, params) = self.destruct();
        ParUseIter {
            using,
            iter,
            xap,
            exe: exe.with_diagnostics(),
            params,
        }
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
    ) -> impl ParUse<Item = Q, Use = Self::Use, Xap = MapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, Self::Item) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = InsOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, &Self::Item) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = FilOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, &Self::Item) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Q, Use = Self::Use, Xap = FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, Self::Item) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUse<
        Item = V::Item,
        Use = Self::Use,
        Xap = FlatMapOf<Self::Xap, V, H>,
        Input = Self::Input,
    >
    where
        V: IntoIterator,
        H: Fn(&mut Self::Use, Self::Item) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        self.with_xap(xap)
    }

    fn flatten(
        self,
    ) -> impl ParUse<
        Item = <Self::Item as IntoIterator>::Item,
        Use = Self::Use,
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
        let (u, iter, x, mut exe, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(params, u, iter, x).map(|x| x.val),
            IterationOrder::Arbitrary => exe.next_any(params, u, iter, x),
        }
    }

    fn reduce<F>(self, f: F) -> Option<X::O>
    where
        F: Fn(&mut X::U, X::O, X::O) -> X::O + Send + Copy,
        X::O: Send,
    {
        let (u, iter, x, mut exe, params) = self.destruct();
        exe.reduce(params, u, iter, x, f)
    }

    fn collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_use_col_into(dst, self),
            IterationOrder::Arbitrary => C::inf_use_arb_col_into(dst, self),
        }
    }

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        let mut dst = C::new_empty();
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_use_col_into(&mut dst, self),
            IterationOrder::Arbitrary => C::inf_use_arb_col_into(&mut dst, self),
        }
        dst
    }
}

impl<U, I, X, R> ParInfCommon for ParUseIter<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    type CommonItem = <Self as ParUseCore>::Item;

    fn common_collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::CommonItem>,
        Self::CommonItem: Send,
    {
        self.collect_into(dst);
    }
}
