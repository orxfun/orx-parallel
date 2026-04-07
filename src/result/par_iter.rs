use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::sizes::Size;
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, Xap};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::result::par_runner::ParRunnerResult;
use crate::result::size_pairs::SizePair;
use crate::result::xap_res::XapRes;
use crate::runner::{DefaultRunner, ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParRes<I, M, E, X1, X2, S, R = DefaultRunner>
where
    I: ConcurrentIter,
    X1: Xap<O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    iter: I,
    xap: XapRes<M, E, X1, X2, S>,
    exe: R,
    params: Params,
}

impl<I, M, E, X1, X2, S, R> ParRes<I, M, E, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub(crate) fn new(iter: I, xap: XapRes<M, E, X1, X2, S>, exe: R, params: Params) -> Self {
        Self {
            iter,
            xap,
            exe,
            params,
        }
    }

    fn with_xap<Y, T>(self, xap: XapRes<M, E, X1, Y, T>) -> ParRes<I, M, E, X1, Y, T, R>
    where
        Y: Xap<I = M>,
        T: SizePair<S1 = X1::Size, S2 = Y::Size>,
    {
        ParRes::new(self.iter, xap, self.exe, self.params)
    }

    fn destruct(self) -> (I, XapRes<M, E, X1, X2, S>, R, Params) {
        (self.iter, self.xap, self.exe, self.params)
    }

    // transformations

    pub fn map<Q, H>(self, h: H) -> ParRes<I, M, E, X1, MapOf<X2, Q, H>, S, R>
    where
        H: Fn(X2::O) -> Q + Copy + Send,
    {
        let xap = self.xap.map(h);
        self.with_xap(xap)
    }

    pub fn inspect<H>(self, h: H) -> ParRes<I, M, E, X1, InsOf<X2, H>, S, R>
    where
        H: Fn(&X2::O) + Copy + Send,
    {
        let xap = self.xap.inspect(h);
        self.with_xap(xap)
    }

    pub fn filter<H>(self, h: H) -> ParRes<I, M, E, X1, FilOf<X2, H>, S::ThenBin, R>
    where
        H: Fn(&X2::O) -> bool + Copy + Send,
    {
        let xap = self.xap.filter(h);
        self.with_xap(xap)
    }

    pub fn filter_map<Q, H>(self, h: H) -> ParRes<I, M, E, X1, FilMapOf<X2, Q, H>, S::ThenBin, R>
    where
        H: Fn(X2::O) -> Option<Q> + Copy + Send,
    {
        let xap = self.xap.filter_map(h);
        self.with_xap(xap)
    }

    pub fn flat_map<V, H>(self, h: H) -> ParRes<I, M, E, X1, FlatMapOf<X2, V, H>, S::ThenMany, R>
    where
        V: IntoIterator,
        H: Fn(X2::O) -> V + Copy + Send,
    {
        let xap = self.xap.flat_map(h);
        self.with_xap(xap)
    }

    // compute

    pub fn first(self) -> Result<Option<X2::O>, E>
    where
        X2::O: Send,
        E: Send,
    {
        let (iter, x, mut exe, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(params, iter, x).map(|x| x.map(|x| x.val)),
            IterationOrder::Arbitrary => exe.next_any(params, iter, x),
        }
    }

    pub fn reduce<F>(self, f: F) -> Result<Option<X2::O>, E>
    where
        F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
        E: Send,
    {
        let (iter, x, mut exe, params) = self.destruct();
        exe.reduce(params, iter, x, f)
    }

    // compute - derived

    pub fn for_each<F>(self, f: F)
    where
        F: Fn(X2::O) + Send + Copy,
        E: Send,
    {
        let _ = self.map(f).reduce(|_, _| {});
    }
}
