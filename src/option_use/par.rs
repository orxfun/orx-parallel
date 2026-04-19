#![allow(refining_impl_trait)]

use crate::ParCollectInto;
use crate::infallible_use::fun::{FnCloned, FnCopied};
use crate::infallible_use::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, Use, XapUse};
use crate::option_use::par_iter::ParUseOptIter;
use crate::option_use::par_runner::ParRunnerUseOpt;
use crate::option_use::size_pairs::SizePairUseOpt;
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner, WithDiagnostics};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParUseOpt<U, I, M, X1, X2, S, R = DefaultRunner>
where
    U: Use,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePairUseOpt<S1 = X1::Size, S2 = X2::Size>,
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

impl<U, I, M, X1, X2, S, R> ParUseOpt<U, I, M, X1, X2, S, R>
where
    U: Use,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePairUseOpt<S1 = X1::Size, S2 = X2::Size>,
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

    fn with_xap2<Y2, T>(self, x2: Y2) -> ParUseOpt<U, I, M, X1, Y2, T, R>
    where
        Y2: XapUse<U = X1::U, I = M>,
        T: SizePairUseOpt<S1 = X1::Size, S2 = Y2::Size>,
    {
        ParUseOpt::new(self.using, self.iter, self.x1, x2, self.exe, self.params)
    }

    pub(crate) fn destruct(self) -> (U, I, X1, X2, R, S, Params) {
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

impl<U, I, M, X1, X2, S, R> ParUseOptIter for ParUseOpt<U, I, M, X1, X2, S, R>
where
    U: Use,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePairUseOpt<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    type Runner = R;

    type Size = S;

    type Use = U;

    type Item = X2::O;

    fn runner<Q: ParRunner>(self, runner: Q) -> ParUseOpt<U, I, M, X1, X2, S, Q> {
        let (using, iter, x1, x2, _, s, params) = self.destruct();
        ParUseOpt {
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
    fn runner_with_diagnostics(self) -> ParUseOpt<U, I, M, X1, X2, S, WithDiagnostics<R>> {
        let (using, iter, x1, x2, exe, s, params) = self.destruct();
        ParUseOpt {
            using,
            iter,
            x1,
            x2,
            exe: exe.with_diagnostics(),
            s,
            params,
        }
    }

    fn num_threads(
        mut self,
        num_threads: impl Into<NumThreads>,
    ) -> ParUseOpt<U, I, M, X1, X2, S, R> {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> ParUseOpt<U, I, M, X1, X2, S, R> {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    fn iteration_order(
        mut self,
        iteration_order: IterationOrder,
    ) -> ParUseOpt<U, I, M, X1, X2, S, R> {
        self.params = self.params.with_collect_ordering(iteration_order);
        self
    }

    // transformations

    fn map<Q, H>(self, h: H) -> ParUseOpt<U, I, M, X1, MapOf<X2, Q, H>, S, R>
    where
        H: Fn(&mut X1::U, X2::O) -> Q + Copy + Send,
    {
        let x2 = self.x2.map(h);
        self.with_xap2(x2)
    }

    fn inspect<H>(self, h: H) -> ParUseOpt<U, I, M, X1, InsOf<X2, H>, S, R>
    where
        H: Fn(&mut X1::U, &X2::O) + Copy + Send,
    {
        let x2 = self.x2.inspect(h);
        self.with_xap2(x2)
    }

    fn filter<H>(self, h: H) -> ParUseOpt<U, I, M, X1, FilOf<X2, H>, S::ThenBin, R>
    where
        H: Fn(&mut X1::U, &X2::O) -> bool + Copy + Send,
        S::ThenBin: SizePairUseOpt,
    {
        let x2 = self.x2.filter(h);
        self.with_xap2(x2)
    }

    fn filter_map<Q, H>(self, h: H) -> ParUseOpt<U, I, M, X1, FilMapOf<X2, Q, H>, S::ThenBin, R>
    where
        H: Fn(&mut X1::U, X2::O) -> Option<Q> + Copy + Send,
        S::ThenBin: SizePairUseOpt,
    {
        let x2 = self.x2.filter_map(h);
        self.with_xap2(x2)
    }

    fn flat_map<V, H>(self, h: H) -> ParUseOpt<U, I, M, X1, FlatMapOf<X2, V, H>, S::ThenMany, R>
    where
        V: IntoIterator,
        H: Fn(&mut X1::U, X2::O) -> V + Copy + Send,
        S::ThenMany: SizePairUseOpt,
    {
        let x2 = self.x2.flat_map(h);
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

    fn collect_into<C>(self, dst: C) -> Option<C>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::opt_use_col_into(Some(dst), self),
            IterationOrder::Arbitrary => C::opt_use_arb_col_into(Some(dst), self),
        }
    }

    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::opt_use_col_into(None, self),
            IterationOrder::Arbitrary => C::opt_use_arb_col_into(None, self),
        }
    }
}

// transformations

impl<'a, U, O, I, M, X1, X2, S, R> ParUseOpt<U, I, M, X1, X2, S, R>
where
    U: Use,
    O: 'a + Copy,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M, O = &'a O>,
    S: SizePairUseOpt<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn copied(self) -> ParUseOpt<U, I, M, X1, MappedOf<X2, FnCopied<'a, U::Item, O>>, S, R> {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseOpt::new(u, iter, x1, x2.mapped(FnCopied::new()), exe, params)
    }
}

impl<'a, U, O, I, M, X1, X2, S, R> ParUseOpt<U, I, M, X1, X2, S, R>
where
    U: Use,
    O: 'a + Clone,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Option<M>>,
    X2: XapUse<U = U::Item, I = M, O = &'a O>,
    S: SizePairUseOpt<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn cloned(self) -> ParUseOpt<U, I, M, X1, MappedOf<X2, FnCloned<'a, U::Item, O>>, S, R> {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseOpt::new(u, iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }
}
