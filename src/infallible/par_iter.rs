use crate::ParCollectInto;
use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::par_runner::ParRunnerInfallible;
use crate::infallible::xap::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf};
use crate::infallible::{Xap, XapEnumByInput};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner};
use crate::sizes::Size;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_iter::enumerate::Enumerate;

pub struct Par<I, X, R = DefaultRunner>
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

impl<I, X, R> Par<I, X, R>
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

    pub(super) fn with_xap<Y: Xap<I = I::Item>>(self, xap: Y) -> Par<I, Y, R> {
        Par::new(self.iter, xap, self.exe, self.params)
    }

    pub(crate) fn destruct(self) -> (I, X, R, Params) {
        (self.iter, self.xap, self.exe, self.params)
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

    pub fn map<Q, H>(self, h: H) -> Par<I, MapOf<X, Q, H>, R>
    where
        H: Fn(X::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    pub fn inspect<H>(self, h: H) -> Par<I, InsOf<X, H>, R>
    where
        H: Fn(&X::O) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    pub fn filter<H>(self, h: H) -> Par<I, FilOf<X, H>, R>
    where
        H: Fn(&X::O) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    pub fn filter_map<Q, H>(self, h: H) -> Par<I, FilMapOf<X, Q, H>, R>
    where
        H: Fn(X::O) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    pub fn flat_map<V, H>(self, h: H) -> Par<I, FlatMapOf<X, V, H>, R>
    where
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        self.with_xap(xap)
    }

    // compute

    pub fn first(self) -> Option<X::O>
    where
        X::O: Send,
    {
        let (iter, x, mut exe, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(params, iter, x).map(|x| x.val),
            IterationOrder::Arbitrary => exe.next_any(params, iter, x),
        }
    }

    pub fn reduce<F>(self, f: F) -> Option<X::O>
    where
        F: Fn(X::O, X::O) -> X::O + Send + Copy,
        X::O: Send,
    {
        let (iter, x, mut exe, params) = self.destruct();
        exe.reduce(params, iter, x, f)
    }

    pub fn collect_into<C>(self, dst: C) -> C
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_col_into(Some(dst), self),
            IterationOrder::Arbitrary => {
                let exact_len = match (self.iter.try_get_len(), <X::Size as Size>::size()) {
                    (Some(input_len), Some(elem_len)) => Some(input_len * elem_len),
                    _ => None,
                };
                C::inf_arb_col_into(Some(dst), self, exact_len)
            }
        }
    }

    pub fn collect<C>(self) -> C
    where
        C: ParCollectInto<X::O>,
        X::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::inf_col_into(None, self),
            IterationOrder::Arbitrary => {
                let exact_len = match (self.iter.try_get_len(), <X::Size as Size>::size()) {
                    (Some(input_len), Some(elem_len)) => Some(input_len * elem_len),
                    _ => None,
                };
                C::inf_arb_col_into(None, self, exact_len)
            }
        }
    }

    // compute - derived

    pub fn for_each<F>(self, f: F)
    where
        F: Fn(X::O) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _| {});
    }
}

// transformations

impl<'a, O: Copy + 'a, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn copied(self) -> Par<I, MappedOf<X, FnCopied<'a, O>>, R> {
        let (iter, xap, exe, params) = self.destruct();
        Par::new(iter, xap.mapped(FnCopied::new()), exe, params)
    }
}

impl<'a, O: Clone + 'a, I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: Xap<I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn cloned(self) -> Par<I, MappedOf<X, FnCloned<'a, O>>, R> {
        let (iter, xap, exe, params) = self.destruct();
        Par::new(iter, xap.mapped(FnCloned::new()), exe, params)
    }
}

impl<I, X, R> Par<I, X, R>
where
    I: ConcurrentIter,
    X: XapEnumByInput<I = I::Item>,
    R: ParRunner,
{
    pub fn enumerate(self) -> Par<Enumerate<I>, X::Enumerated, R> {
        let (iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        Par::new(iter, xap, exe, params)
    }
}
