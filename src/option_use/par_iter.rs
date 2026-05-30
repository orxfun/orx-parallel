#![allow(refining_impl_trait)]

use crate::ParCollectInto;
use crate::common_par_traits::ParOptCommon;
use crate::infallible_use::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, Using, XapUse};
use crate::option_use::par::ParUseOption;
use crate::option_use::par_core::ParUseOptionCore;
use crate::option_use::par_runner::ParRunnerUseOpt;
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::pool::ParThreadPool;
use crate::runner::{DefaultRunner, ParRunner};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub struct ParUseOptionIter<U, I, M, X1, X2, S, R = DefaultRunner>
where
    U: Using,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    using: U,
    iter: I,
    x1: X1,
    x2: X2,
    exe: R,
    params: Params,
    s: S,
}

impl<U, I, M, X1, X2, S, R> ParUseOptionIter<U, I, M, X1, X2, S, R>
where
    U: Using,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub(crate) fn new(using: U, iter: I, x1: X1, x2: X2, exe: R, params: Params) -> Self {
        Self {
            using,
            iter,
            x1,
            x2,
            exe,
            params,
            s: Default::default(),
        }
    }

    fn with_xap2<Y2, T>(self, x2: Y2) -> ParUseOptionIter<U, I, M, X1, Y2, T, R>
    where
        Y2: XapUse<U = X1::U, I = M>,
        T: SizePair<S1 = X1::Size, S2 = Y2::Size>,
    {
        ParUseOptionIter::new(self.using, self.iter, self.x1, x2, self.exe, self.params)
    }
}

impl<U, I, M, X1, X2, S, R> ParUseOptionCore for ParUseOptionIter<U, I, M, X1, X2, S, R>
where
    U: Using,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    type Item = X2::O;

    type Runner = R;

    type Use = U::Item;

    type Using = U;

    type Input = I;

    type M = M;

    type Xap1 = X1;

    type Xap2 = X2;

    type Size = S;

    fn destruct(
        self,
    ) -> (
        Self::Using,
        Self::Input,
        Self::Xap1,
        Self::Xap2,
        Self::Runner,
        Self::Size,
        Params,
    ) {
        (
            self.using,
            self.iter,
            self.x1,
            self.x2,
            self.exe,
            self.s,
            self.params,
        )
    }
}

impl<U, I, M, X1, X2, S, R> ParUseOption for ParUseOptionIter<U, I, M, X1, X2, S, R>
where
    U: Using,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    > {
        let (using, iter, x1, x2, _, s, params) = self.destruct();
        ParUseOptionIter {
            using,
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
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    > {
        let (using, iter, x1, x2, exe, s, params) = self.destruct();
        ParUseOptionIter {
            using,
            iter,
            x1,
            x2,
            exe: exe.with_diagnostics(),
            s,
            params,
        }
    }

    fn pool<P: ParThreadPool>(
        self,
        pool: P,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    > {
        let (using, iter, x1, x2, exe, s, params) = self.destruct();
        let exe = exe.with_pool(pool);
        ParUseOptionIter {
            using,
            iter,
            x1,
            x2,
            exe,
            s,
            params,
        }
    }

    fn num_threads(
        mut self,
        num_threads: impl Into<NumThreads>,
    ) -> ParUseOptionIter<U, I, M, X1, X2, S, R> {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(
        mut self,
        chunk_size: impl Into<ChunkSize>,
    ) -> ParUseOptionIter<U, I, M, X1, X2, S, R> {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    fn iteration_order(
        mut self,
        iteration_order: IterationOrder,
    ) -> ParUseOptionIter<U, I, M, X1, X2, S, R> {
        self.params = self.params.with_collect_ordering(iteration_order);
        self
    }

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = Q,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&mut X1::U, X2::O) -> Q + Copy + Send,
    {
        let x2 = self.x2.map(h);
        self.with_xap2(x2)
    }

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&mut X1::U, &X2::O) + Copy + Send,
    {
        let x2 = self.x2.inspect(h);
        self.with_xap2(x2)
    }

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&mut X1::U, &X2::O) -> bool + Copy + Send,
    {
        let x2 = self.x2.filter(h);
        self.with_xap2(x2)
    }

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = Q,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&mut X1::U, X2::O) -> Option<Q> + Copy + Send,
    {
        let x2 = self.x2.filter_map(h);
        self.with_xap2(x2)
    }

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Item = V::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
    >
    where
        V: IntoIterator,
        H: Fn(&mut X1::U, X2::O) -> V + Copy + Send,
    {
        let x2 = self.x2.flat_map(h);
        self.with_xap2(x2)
    }

    fn flatten(
        self,
    ) -> impl ParUseOption<
        Item = <Self::Item as IntoIterator>::Item,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlattenOf<Self::Xap2>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
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
        let (u, iter, x1, x2, mut exe, s, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe
                .next(s, params, u, iter, x1, x2)
                .map(|x| x.map(|x| x.val)),
            IterationOrder::Arbitrary => exe.next_any(s, params, u, iter, x1, x2),
        }
    }

    fn reduce<F>(self, f: F) -> Option<Option<X2::O>>
    where
        F: Fn(&mut X1::U, X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = self.destruct();
        exe.reduce(s, params, u, iter, x1, x2, f)
    }

    fn collect_into<C>(self, dst: &mut C) -> Option<()>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::opt_use_col_into(dst, self),
            IterationOrder::Arbitrary => C::opt_use_arb_col_into(dst, self),
        }
    }

    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
    {
        let mut dst = C::new_empty();
        match self.params.iteration_order {
            IterationOrder::Ordered => C::opt_use_col_into(&mut dst, self),
            IterationOrder::Arbitrary => C::opt_use_arb_col_into(&mut dst, self),
        }
        .map(|_| dst)
    }
}

impl<U, I, M, X1, X2, S, R> ParOptCommon for ParUseOptionIter<U, I, M, X1, X2, S, R>
where
    U: Using,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    type CommonItem = <Self as ParUseOptionCore>::Item;

    fn common_collect_into<C>(self, dst: &mut C) -> Option<()>
    where
        C: ParCollectInto<Self::CommonItem>,
        Self::CommonItem: Send,
    {
        self.collect_into(dst)
    }
}
