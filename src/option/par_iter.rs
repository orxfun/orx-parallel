#![allow(refining_impl_trait)]

use crate::ParCollectInto;
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, Xap};
use crate::option::ParOptionCore;
use crate::option::par::ParOption;
use crate::option::par_runner::ParRunnerOpt;
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub struct ParOptionIter<I, M, X1, X2, S, R = DefaultRunner>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    iter: I,
    x1: X1,
    x2: X2,
    exe: R,
    params: Params,
    s: S,
}

impl<I, M, X1, X2, S, R> ParOptionIter<I, M, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub(crate) fn new(iter: I, x1: X1, x2: X2, exe: R, params: Params) -> Self {
        Self {
            iter,
            x1,
            x2,
            exe,
            params,
            s: Default::default(),
        }
    }

    fn with_xap2<Y2, T>(self, x2: Y2) -> ParOptionIter<I, M, X1, Y2, T, R>
    where
        Y2: Xap<I = M>,
        T: SizePair<S1 = X1::Size, S2 = Y2::Size>,
    {
        ParOptionIter::new(self.iter, self.x1, x2, self.exe, self.params)
    }
}

impl<I, M, X1, X2, S, R> ParOptionCore for ParOptionIter<I, M, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    type Item = X2::O;

    type Runner = R;

    type Input = I;

    type M = M;

    type Xap1 = X1;

    type Xap2 = X2;

    type Size = S;

    fn destruct(
        self,
    ) -> (
        Self::Input,
        Self::Xap1,
        Self::Xap2,
        Self::Runner,
        Self::Size,
        Params,
    ) {
        (self.iter, self.x1, self.x2, self.exe, self.s, self.params)
    }
}

impl<I, M, X1, X2, S, R> ParOption for ParOptionIter<I, M, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParOption<
        Input = Self::Input,
        Size = Self::Size,
        Xap1 = Self::Xap1,
        Xap2 = Self::Xap2,
        M = Self::M,
        Item = Self::Item,
    > {
        let (iter, x1, x2, _, s, params) = self.destruct();
        ParOptionIter {
            iter,
            x1,
            x2,
            exe: runner,
            s,
            params,
        }
    }

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParOption<
        Input = Self::Input,
        Size = Self::Size,
        Xap1 = Self::Xap1,
        Xap2 = Self::Xap2,
        M = Self::M,
        Item = Self::Item,
    > {
        let (iter, x1, x2, exe, s, params) = self.destruct();
        ParOptionIter {
            iter,
            x1,
            x2,
            exe: exe.with_diagnostics(),
            s,
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
    ) -> impl ParOption<
        Item = Q,
        Input = Self::Input,
        Size = Self::Size,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = MapOf<Self::Xap2, Q, H>,
    >
    where
        H: Fn(X2::O) -> Q + Copy + Send,
    {
        let x2 = self.x2.map(h);
        self.with_xap2(x2)
    }

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Self::Item,
        Input = Self::Input,
        Size = Self::Size,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = InsOf<Self::Xap2, H>,
    >
    where
        H: Fn(&X2::O) + Copy + Send,
    {
        let x2 = self.x2.inspect(h);
        self.with_xap2(x2)
    }

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Self::Item,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = FilOf<Self::Xap2, H>,
    >
    where
        H: Fn(&X2::O) -> bool + Copy + Send,
    {
        let x2 = self.x2.filter(h);
        self.with_xap2(x2)
    }

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Q,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
    >
    where
        H: Fn(X2::O) -> Option<Q> + Copy + Send,
    {
        let x2 = self.x2.filter_map(h);
        self.with_xap2(x2)
    }

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = V::Item,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
    >
    where
        V: IntoIterator,
        H: Fn(X2::O) -> V + Copy + Send,
    {
        let x2 = self.x2.flat_map(h);
        self.with_xap2(x2)
    }

    fn flatten(
        self,
    ) -> impl ParOption<
        Runner = Self::Runner,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = FlattenOf<Self::Xap2>,
        Item = <Self::Item as IntoIterator>::Item,
    >
    where
        Self::Item: IntoIterator,
    {
        let x2 = self.x2.flatten();
        self.with_xap2(x2)
    }

    // compute

    fn first(self) -> Option<Option<X2::O>>
    where
        X2::O: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(s, params, iter, x1, x2).map(|x| x.map(|x| x.val)),
            IterationOrder::Arbitrary => exe.next_any(s, params, iter, x1, x2),
        }
    }

    fn reduce<F>(self, f: F) -> Option<Option<X2::O>>
    where
        F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = self.destruct();
        exe.reduce(s, params, iter, x1, x2, f)
    }

    fn collect_into<C>(self, dst: C) -> Option<C>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::opt_col_into(Some(dst), self),
            IterationOrder::Arbitrary => C::opt_arb_col_into(Some(dst), self),
        }
    }

    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::opt_col_into(None, self),
            IterationOrder::Arbitrary => C::opt_arb_col_into(None, self),
        }
    }
}
