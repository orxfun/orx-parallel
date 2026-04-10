use crate::ParCollectInto;
use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, Xap};
use crate::option::par_runner::ParRunnerOpt;
use crate::option::size_pairs::SizePairOpt;
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::runner::{DefaultRunner, ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParOpt<I, M, X1, X2, S, R = DefaultRunner>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    iter: I,
    x1: X1,
    x2: X2,
    exe: R,
    params: Params,
    s: S,
}

impl<I, M, X1, X2, S, R> ParOpt<I, M, X1, X2, S, R>
where
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M>,
    S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
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

    fn with_xap2<Y2, T>(self, x2: Y2) -> ParOpt<I, M, X1, Y2, T, R>
    where
        Y2: Xap<I = M>,
        T: SizePairOpt<S1 = X1::Size, S2 = Y2::Size>,
    {
        ParOpt::new(self.iter, self.x1, x2, self.exe, self.params)
    }

    pub(crate) fn destruct(self) -> (I, X1, X2, R, S, Params) {
        (self.iter, self.x1, self.x2, self.exe, self.s, self.params)
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

    pub fn map<Q, H>(self, h: H) -> ParOpt<I, M, X1, MapOf<X2, Q, H>, S, R>
    where
        H: Fn(X2::O) -> Q + Copy + Send,
    {
        let x2 = self.x2.map(h);
        self.with_xap2(x2)
    }

    pub fn inspect<H>(self, h: H) -> ParOpt<I, M, X1, InsOf<X2, H>, S, R>
    where
        H: Fn(&X2::O) + Copy + Send,
    {
        let x2 = self.x2.inspect(h);
        self.with_xap2(x2)
    }

    pub fn filter<H>(self, h: H) -> ParOpt<I, M, X1, FilOf<X2, H>, S::ThenBin, R>
    where
        H: Fn(&X2::O) -> bool + Copy + Send,
        S::ThenBin: SizePairOpt,
    {
        let x2 = self.x2.filter(h);
        self.with_xap2(x2)
    }

    pub fn filter_map<Q, H>(self, h: H) -> ParOpt<I, M, X1, FilMapOf<X2, Q, H>, S::ThenBin, R>
    where
        H: Fn(X2::O) -> Option<Q> + Copy + Send,
        S::ThenBin: SizePairOpt,
    {
        let x2 = self.x2.filter_map(h);
        self.with_xap2(x2)
    }

    pub fn flat_map<V, H>(self, h: H) -> ParOpt<I, M, X1, FlatMapOf<X2, V, H>, S::ThenMany, R>
    where
        V: IntoIterator,
        H: Fn(X2::O) -> V + Copy + Send,
        S::ThenMany: SizePairOpt,
    {
        let x2 = self.x2.flat_map(h);
        self.with_xap2(x2)
    }

    // compute

    pub fn first(self) -> Option<Option<X2::O>>
    where
        X2::O: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe.next(s, params, iter, x1, x2).map(|x| x.map(|x| x.val)),
            IterationOrder::Arbitrary => exe.next_any(s, params, iter, x1, x2),
        }
    }

    pub fn reduce<F>(self, f: F) -> Option<Option<X2::O>>
    where
        F: Fn(X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
    {
        let (iter, x1, x2, mut exe, s, params) = self.destruct();
        exe.reduce(s, params, iter, x1, x2, f)
    }

    pub fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::opt_col_into(None, self),
            IterationOrder::Arbitrary => {
                // let exact_len = <X::Size as Size>::output_len(self.iter.try_get_len());
                // C::inf_arb_col_into(None, self, exact_len)
                todo!()
            }
        }
    }

    // compute - derived

    pub fn for_each<F>(self, f: F)
    where
        F: Fn(X2::O) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _| {});
    }
}

// transformations

impl<'a, O, I, M, X1, X2, S, R> ParOpt<I, M, X1, X2, S, R>
where
    O: 'a + Copy,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M, O = &'a O>,
    S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn copied(self) -> ParOpt<I, M, X1, MappedOf<X2, FnCopied<'a, O>>, S, R> {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParOpt::new(iter, x1, x2.mapped(FnCopied::new()), exe, params)
    }
}

impl<'a, O, I, M, X1, X2, S, R> ParOpt<I, M, X1, X2, S, R>
where
    O: 'a + Clone,
    I: ConcurrentIter,
    X1: Xap<I = I::Item, O = Option<M>>,
    X2: Xap<I = M, O = &'a O>,
    S: SizePairOpt<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn cloned(self) -> ParOpt<I, M, X1, MappedOf<X2, FnCloned<'a, O>>, S, R> {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParOpt::new(iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }
}
