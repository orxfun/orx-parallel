#![allow(refining_impl_trait)]

use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::par_iter::ParUseIter;
use crate::infallible_use::par_runner::ParRunnerInfallibleUse;
use crate::infallible_use::use_var::Use;
use crate::infallible_use::xap::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf};
use crate::infallible_use::{ParUseIterCore, XapUse, XapUseEnumByInput};
use crate::parameters::{IterationOrder, Params};
use crate::runner::{DefaultRunner, ParRunner};
use crate::{ChunkSize, NumThreads, ParCollectInto};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_iter::enumerate::Enumerate;

pub struct ParUse<U, I, X, R = DefaultRunner>
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

impl<U, I, X, R> ParUse<U, I, X, R>
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
    ) -> ParUse<U, I, Y, R> {
        ParUse::new(self.using, self.iter, xap, self.exe, self.params)
    }

    pub(crate) fn destruct(self) -> (U, I, X, R, Params) {
        (self.using, self.iter, self.xap, self.exe, self.params)
    }
}

impl<U, I, X, R> ParUseIterCore for ParUse<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    type Item = X::O;

    type Runner = R;

    type Use = U;

    type Input = I;

    type Xap = X;

    fn destruct(self) -> (Self::Use, Self::Input, Self::Xap, Self::Runner, Params) {
        todo!()
    }
}

impl<U, I, X, R> ParUseIter for ParUse<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    // configuration

    fn runner<Q: ParRunner>(self, runner: Q) -> ParUse<U, I, X, Q> {
        let (using, iter, xap, _, params) = self.destruct();
        ParUse {
            using,
            iter,
            xap,
            exe: runner,
            params,
        }
    }

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(self) -> ParUse<U, I, X, crate::runner::WithDiagnostics<R>> {
        let (using, iter, xap, exe, params) = self.destruct();
        ParUse {
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

    fn map<Q, H>(self, h: H) -> ParUse<U, I, MapOf<X, Q, H>, R>
    where
        H: Fn(&mut U::Item, X::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    fn inspect<H>(self, h: H) -> ParUse<U, I, InsOf<X, H>, R>
    where
        H: Fn(&mut U::Item, &X::O) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    fn filter<H>(self, h: H) -> ParUse<U, I, FilOf<X, H>, R>
    where
        H: Fn(&mut U::Item, &X::O) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    fn filter_map<Q, H>(self, h: H) -> ParUse<U, I, FilMapOf<X, Q, H>, R>
    where
        H: Fn(&mut U::Item, X::O) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    fn flat_map<V, H>(self, h: H) -> ParUse<U, I, FlatMapOf<X, V, H>, R>
    where
        V: IntoIterator,
        H: Fn(&mut U::Item, X::O) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
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

    fn collect_into<C>(self, dst: C) -> C
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_use_col_into(Some(dst), self),
            IterationOrder::Arbitrary => C::inf_use_arb_col_into(Some(dst), self),
        }
    }

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_use_col_into(None, self),
            IterationOrder::Arbitrary => C::inf_use_arb_col_into(None, self),
        }
    }
}

// transformations

impl<'a, U, O: Copy + 'a, I, X, R> ParUse<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn copied(self) -> ParUse<U, I, MappedOf<X, UFnCopied<'a, U::Item, O>>, R> {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUse::new(u, iter, xap.mapped(UFnCopied::new()), exe, params)
    }
}

impl<'a, U, O: Clone + 'a, I, X, R> ParUse<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUse<U = U::Item, I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn cloned(self) -> ParUse<U, I, MappedOf<X, UFnCloned<'a, U::Item, O>>, R> {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUse::new(u, iter, xap.mapped(UFnCloned::new()), exe, params)
    }
}

impl<U, I, X, R> ParUse<U, I, X, R>
where
    U: Use,
    I: ConcurrentIter,
    X: XapUseEnumByInput<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    pub fn enumerate(self) -> ParUse<U, Enumerate<I>, X::Enumerated, R> {
        let (u, iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        ParUse::new(u, iter, xap, exe, params)
    }
}
