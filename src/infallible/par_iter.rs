use crate::infallible::par_runner::ParRunnerInfallible;
use crate::infallible::xap::Xap;
use crate::infallible::xap_variants::Id;
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner, default_runner};
use orx_concurrent_iter::ConcurrentIter;

// TODO: this will be replaced later by IntoPar trait.
pub fn par<I: ConcurrentIter>(iter: I) -> Par<I, Id<I::Item>, DefaultRunner> {
    Par {
        iter,
        xap: Id::new(),
        exe: default_runner(),
        params: Default::default(),
    }
}

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
    fn new(iter: I, xap: X, exe: R, params: Params) -> Self {
        Self {
            iter,
            xap,
            exe,
            params,
        }
    }

    fn with_xap<Y: Xap<I = I::Item>>(self, xap: Y) -> Par<I, Y, R> {
        Par::new(self.iter, xap, self.exe, self.params)
    }

    fn destruct(self) -> (I, X, R, Params) {
        (self.iter, self.xap, self.exe, self.params)
    }

    // params

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

    pub fn map<Q, H>(self, h: H) -> Par<I, X::Map<Q, H>, R>
    where
        H: Fn(X::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    pub fn inspect<H>(self, h: H) -> Par<I, X::Inspect<H>, R>
    where
        H: Fn(&X::O) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    pub fn filter<H>(self, h: H) -> Par<I, X::Filter<H>, R>
    where
        H: Fn(&X::O) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    pub fn filter_map<Q, H>(self, h: H) -> Par<I, X::FilterMap<Q, H>, R>
    where
        H: Fn(X::O) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    pub fn flat_map<V, H>(self, h: H) -> Par<I, X::FlatMap<V, H>, R>
    where
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        self.with_xap(xap)
    }

    // compute - early exit

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
}
