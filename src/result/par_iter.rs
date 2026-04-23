#![allow(refining_impl_trait)]

use crate::ParCollectInto;
use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, Xap};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::result::par::ParResult;
use crate::result::par_core::ParResultCore;
use crate::result::par_runner::ParRunnerRes;
use crate::runner::{DefaultRunner, ParRunner, WithDiagnostics};
use crate::sizes::SizePair;
use orx_concurrent_iter::ConcurrentIter;

pub struct ParResultIter<I, M, E, X1, X2, S, R = DefaultRunner>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
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

impl<I, M, E, X1, X2, S, R> ParResultIter<I, M, E, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
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

    fn with_xap2<Y2, T>(self, x2: Y2) -> ParResultIter<I, M, E, X1, Y2, T, R>
    where
        Y2: Xap<I = M>,
        T: SizePair<S1 = X1::Size, S2 = Y2::Size>,
    {
        ParResultIter::new(self.iter, self.x1, x2, self.exe, self.params)
    }
}

impl<I, M, E, X1, X2, S, R> ParResultCore for ParResultIter<I, M, E, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    type Item = X2::O;

    type Error = E;

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

impl<I, M, E, X1, X2, S, R> ParResult for ParResultIter<I, M, E, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    fn runner<Q: ParRunner>(self, runner: Q) -> ParResultIter<I, M, E, X1, X2, S, Q> {
        let (iter, x1, x2, _, s, params) = self.destruct();
        ParResultIter {
            iter,
            x1,
            x2,
            exe: runner,
            s,
            params,
        }
    }

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(self) -> ParResultIter<I, M, E, X1, X2, S, WithDiagnostics<R>> {
        let (iter, x1, x2, exe, s, params) = self.destruct();
        ParResultIter {
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

    fn map<Q, H>(self, h: H) -> ParResultIter<I, M, E, X1, MapOf<X2, Q, H>, S, R>
    where
        H: Fn(X2::O) -> Q + Copy + Send,
    {
        let x2 = self.x2.map(h);
        self.with_xap2(x2)
    }

    fn inspect<H>(self, h: H) -> ParResultIter<I, M, E, X1, InsOf<X2, H>, S, R>
    where
        H: Fn(&X2::O) + Copy + Send,
    {
        let x2 = self.x2.inspect(h);
        self.with_xap2(x2)
    }

    fn filter<H>(self, h: H) -> ParResultIter<I, M, E, X1, FilOf<X2, H>, S::ThenBin, R>
    where
        H: Fn(&X2::O) -> bool + Copy + Send,
        S::ThenBin: SizePair,
    {
        let x2 = self.x2.filter(h);
        self.with_xap2(x2)
    }

    fn filter_map<Q, H>(self, h: H) -> ParResultIter<I, M, E, X1, FilMapOf<X2, Q, H>, S::ThenBin, R>
    where
        H: Fn(X2::O) -> Option<Q> + Copy + Send,
        S::ThenBin: SizePair,
    {
        let x2 = self.x2.filter_map(h);
        self.with_xap2(x2)
    }

    fn flat_map<V, H>(self, h: H) -> ParResultIter<I, M, E, X1, FlatMapOf<X2, V, H>, S::ThenMany, R>
    where
        V: IntoIterator,
        H: Fn(X2::O) -> V + Copy + Send,
        S::ThenMany: SizePair,
    {
        let x2 = self.x2.flat_map(h);
        self.with_xap2(x2)
    }

    fn first(self) -> Result<Option<X2::O>, E>
    where
        X2::O: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(s, params, iter, x1, x2).map(|x| x.map(|x| x.val)),
            IterationOrder::Arbitrary => exe.next_any(s, params, iter, x1, x2),
        }
    }

    fn reduce<F>(self, f: F) -> Result<Option<X2::O>, E>
    where
        F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
        E: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = self.destruct();
        exe.reduce(s, params, iter, x1, x2, f)
    }

    fn collect_into<C>(self, dst: C) -> Result<C, E>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
        E: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::res_col_into(Some(dst), self),
            IterationOrder::Arbitrary => C::res_arb_col_into(Some(dst), self),
        }
    }

    fn collect<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
        E: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::res_col_into(None, self),
            IterationOrder::Arbitrary => C::res_arb_col_into(None, self),
        }
    }
}

// transformations

impl<'a, O, I, M, E, X1, X2, S, R> ParResultIter<I, M, E, X1, X2, S, R>
where
    O: 'a + Copy,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M, O = &'a O>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn copied(self) -> ParResultIter<I, M, E, X1, MappedOf<X2, FnCopied<'a, O>>, S, R> {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParResultIter::new(iter, x1, x2.mapped(FnCopied::new()), exe, params)
    }
}

impl<'a, O, I, M, E, X1, X2, S, R> ParResultIter<I, M, E, X1, X2, S, R>
where
    O: 'a + Clone,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M, O = &'a O>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn cloned(self) -> ParResultIter<I, M, E, X1, MappedOf<X2, FnCloned<'a, O>>, S, R> {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParResultIter::new(iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }
}
