#![allow(refining_impl_trait)]

use crate::infallible::Xap;
use crate::infallible::par_core::ParCore;
use crate::infallible::par_runner::ParRunnerInfallible;
use crate::infallible::xap::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner, WithDiagnostics};
use crate::{Par, ParCollectInto};
use orx_concurrent_iter::ConcurrentIter;

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

    fn runner<Q: ParRunner>(self, runner: Q) -> ParIter<I, X, Q> {
        let (iter, xap, _, params) = self.destruct();
        ParIter {
            iter,
            xap,
            exe: runner,
            params,
        }
    }

    fn runner_with_diagnostics(self) -> ParIter<I, X, WithDiagnostics<R>> {
        let (iter, xap, exe, params) = self.destruct();
        ParIter {
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

    fn map<Q, H>(self, h: H) -> ParIter<I, MapOf<X, Q, H>, R>
    where
        H: Fn(X::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    fn inspect<H>(self, h: H) -> ParIter<I, InsOf<X, H>, R>
    where
        H: Fn(&X::O) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    fn filter<H>(self, h: H) -> ParIter<I, FilOf<X, H>, R>
    where
        H: Fn(&X::O) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    fn filter_map<Q, H>(self, h: H) -> ParIter<I, FilMapOf<X, Q, H>, R>
    where
        H: Fn(X::O) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    fn flat_map<V, H>(self, h: H) -> ParIter<I, FlatMapOf<X, V, H>, R>
    where
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
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

    fn collect_into<C>(self, dst: C) -> C
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_col_into(Some(dst), self),
            IterationOrder::Arbitrary => C::inf_arb_col_into(Some(dst), self),
        }
    }

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_col_into(None, self),
            IterationOrder::Arbitrary => C::inf_arb_col_into(None, self),
        }
    }
}
