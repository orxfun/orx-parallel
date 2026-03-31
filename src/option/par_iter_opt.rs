use crate::infallible::fun::{FnCloned, FnCopied};
use crate::option::par_runner::ParRunnerOption;
use crate::option::xap_opt::XapOpt;
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParOpt<I, X, R = DefaultRunner>
where
    I: ConcurrentIter,
    X: XapOpt<I = I::Item>,
    R: ParRunner,
{
    iter: I,
    xap: X,
    exe: R,
    params: Params,
}

impl<I, X, R> ParOpt<I, X, R>
where
    I: ConcurrentIter,
    X: XapOpt<I = I::Item>,
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

    fn with_xap<Y: XapOpt<I = I::Item>>(self, xap: Y) -> ParOpt<I, Y, R> {
        ParOpt::new(self.iter, xap, self.exe, self.params)
    }

    fn destruct(self) -> (I, X, R, Params) {
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

    pub fn map<Q, H>(self, h: H) -> ParOpt<I, X::Map<Q, H>, R>
    where
        H: Fn(X::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    pub fn inspect<H>(self, h: H) -> ParOpt<I, X::Inspect<H>, R>
    where
        H: Fn(&X::O) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    pub fn filter<H>(self, h: H) -> ParOpt<I, X::Filter<H>, R>
    where
        H: Fn(&X::O) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    pub fn filter_map<Q, H>(self, h: H) -> ParOpt<I, X::FilterMap<Q, H>, R>
    where
        H: Fn(X::O) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    pub fn flat_map<V, H>(self, h: H) -> ParOpt<I, X::FlatMap<V, H>, R>
    where
        V: IntoIterator,
        H: Fn(X::O) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        self.with_xap(xap)
    }

    // compute

    // TODO: needs good documentation
    pub fn first(self) -> Option<Option<X::O>>
    where
        X::O: Send,
    {
        let (iter, x, mut exe, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(params, iter, x).map(|x| x.map(|x| x.val)),
            IterationOrder::Arbitrary => exe.next_any(params, iter, x),
        }
    }

    pub fn reduce<F>(self, f: F) -> Option<Option<X::O>>
    where
        F: Fn(X::O, X::O) -> X::O + Send + Copy,
        X::O: Send,
    {
        let (iter, x, mut exe, params) = self.destruct();
        exe.reduce(params, iter, x, f)
    }
}

// transformations

impl<'a, O: Copy + 'a, I, X, R> ParOpt<I, X, R>
where
    I: ConcurrentIter,
    X: XapOpt<I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn copied(self) -> ParOpt<I, X::Mapped<FnCopied<'a, O>>, R> {
        let (iter, xap, exe, params) = self.destruct();
        ParOpt::new(iter, xap.mapped(FnCopied::new()), exe, params)
    }
}

impl<'a, O: Clone + 'a, I, X, R> ParOpt<I, X, R>
where
    I: ConcurrentIter,
    X: XapOpt<I = I::Item, O = &'a O>,
    R: ParRunner,
{
    pub fn cloned(self) -> ParOpt<I, X::Mapped<FnCloned<'a, O>>, R> {
        let (iter, xap, exe, params) = self.destruct();
        ParOpt::new(iter, xap.mapped(FnCloned::new()), exe, params)
    }
}
