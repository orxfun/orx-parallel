use crate::infallible_using::fun::{FnCloned, FnCopied};
use crate::infallible_using::par_runner::ParRunnerInfallibleUsing;
use crate::infallible_using::using_var::Using;
use crate::infallible_using::xap::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf};
use crate::infallible_using::{Xap, XapEnumByInput};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_iter::enumerate::Enumerate;

pub struct ParUse<U, I, X, R = DefaultRunner>
where
    U: Using,
    I: ConcurrentIter,
    X: Xap<U = U::Item, I = I::Item>,
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
    U: Using,
    I: ConcurrentIter,
    X: Xap<U = U::Item, I = I::Item>,
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

    pub(super) fn with_xap<Y: Xap<U = U::Item, I = I::Item>>(self, xap: Y) -> ParUse<U, I, Y, R> {
        ParUse::new(self.using, self.iter, xap, self.exe, self.params)
    }

    pub(crate) fn destruct(self) -> (U, I, X, R, Params) {
        (self.using, self.iter, self.xap, self.exe, self.params)
    }

    // params

    pub fn num_threads(mut self, num_threads: impl Into<NumThreads>) -> Self {
        self.params = self.params.with_num_threads(num_threads);
        self
    }

    pub fn chunk_size(mut self, chunk_size: impl Into<ChunkSize>) -> Self {
        self.params = self.params.with_chunk_size(chunk_size);
        self
    }

    pub fn iteration_order(mut self, collect: IterationOrder) -> Self {
        self.params = self.params.with_collect_ordering(collect);
        self
    }

    // transformations

    pub fn map<Q, H>(self, h: H) -> ParUse<U, I, MapOf<X, Q, H>, R>
    where
        H: Fn(&mut U::Item, X::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    pub fn inspect<H>(self, h: H) -> ParUse<U, I, InsOf<X, H>, R>
    where
        H: Fn(&mut U::Item, &X::O) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    pub fn filter<H>(self, h: H) -> ParUse<U, I, FilOf<X, H>, R>
    where
        H: Fn(&mut U::Item, &X::O) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    pub fn filter_map<Q, H>(self, h: H) -> ParUse<U, I, FilMapOf<X, Q, H>, R>
    where
        H: Fn(&mut U::Item, X::O) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    pub fn flat_map<V, H>(self, h: H) -> ParUse<U, I, FlatMapOf<X, V, H>, R>
    where
        V: IntoIterator,
        H: Fn(&mut U::Item, X::O) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        self.with_xap(xap)
    }

    // compute

    pub fn first(self) -> Option<X::O>
    where
        X::O: Send,
    {
        let (u, iter, x, mut exe, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(params, u, iter, x).map(|x| x.val),
            IterationOrder::Arbitrary => exe.next_any(params, u, iter, x),
        }
    }

    pub fn reduce<F>(self, f: F) -> Option<X::O>
    where
        F: Fn(X::O, X::O) -> X::O + Send + Copy,
        X::O: Send,
    {
        let (u, iter, x, mut exe, params) = self.destruct();
        exe.reduce(params, u, iter, x, f)
    }

    // compute - derived

    pub fn for_each<F>(self, f: F)
    where
        F: Fn(&mut U::Item, X::O) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _| {});
    }
}

// transformations

impl<'a, U, O: Copy + 'a, I, X, R> ParUse<U, I, X, R>
where
    U: Using,
    I: ConcurrentIter,
    X: Xap<U = U::Item, I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn copied(self) -> ParUse<U, I, MappedOf<X, FnCopied<'a, U::Item, O>>, R> {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUse::new(u, iter, xap.mapped(FnCopied::new()), exe, params)
    }
}

impl<'a, U, O: Clone + 'a, I, X, R> ParUse<U, I, X, R>
where
    U: Using,
    I: ConcurrentIter,
    X: Xap<U = U::Item, I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn cloned(self) -> ParUse<U, I, MappedOf<X, FnCloned<'a, U::Item, O>>, R> {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUse::new(u, iter, xap.mapped(FnCloned::new()), exe, params)
    }
}

impl<U, I, X, R> ParUse<U, I, X, R>
where
    U: Using,
    I: ConcurrentIter,
    X: XapEnumByInput<U = U::Item, I = I::Item>,
    R: ParRunner,
{
    pub fn enumerate(self) -> ParUse<U, Enumerate<I>, X::Enumerated, R> {
        let (u, iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        ParUse::new(u, iter, xap, exe, params)
    }
}
