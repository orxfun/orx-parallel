use crate::ParCollectInto;
use crate::infallible_use::fun::{FnCloned, FnCopied};
use crate::infallible_use::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, Use, XapUse};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::result_use::par_runner::ParRunnerUseRes;
use crate::result_use::size_pairs::SizePairUseRes;
use crate::runner::{DefaultRunner, ParRunner, RunnerWithDiagnostics};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParUseRes<U, I, M, E, X1, X2, S, R = DefaultRunner>
where
    U: Use,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
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

impl<U, I, M, E, X1, X2, S, R> ParUseRes<U, I, M, E, X1, X2, S, R>
where
    U: Use,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M>,
    S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
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

    fn with_xap2<Y2, T>(self, x2: Y2) -> ParUseRes<U, I, M, E, X1, Y2, T, R>
    where
        Y2: XapUse<U = X1::U, I = M>,
        T: SizePairUseRes<S1 = X1::Size, S2 = Y2::Size>,
    {
        ParUseRes::new(self.using, self.iter, self.x1, x2, self.exe, self.params)
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
    // params

    pub fn runner<Q: ParRunner>(self, runner: Q) -> ParUseRes<U, I, M, E, X1, X2, S, Q> {
        let (using, iter, x1, x2, _, s, params) = self.destruct();
        ParUseRes {
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
    pub fn runner_with_diagnostics(
        self,
    ) -> ParUseRes<U, I, M, E, X1, X2, S, RunnerWithDiagnostics<R>> {
        let (using, iter, x1, x2, exe, s, params) = self.destruct();
        ParUseRes {
            using,
            iter,
            x1,
            x2,
            exe: exe.with_diagnostics(),
            s,
            params,
        }
    }

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

    pub fn map<Q, H>(self, h: H) -> ParUseRes<U, I, M, E, X1, MapOf<X2, Q, H>, S, R>
    where
        H: Fn(&mut X1::U, X2::O) -> Q + Copy + Send,
    {
        let x2 = self.x2.map(h);
        self.with_xap2(x2)
    }

    pub fn inspect<H>(self, h: H) -> ParUseRes<U, I, M, E, X1, InsOf<X2, H>, S, R>
    where
        H: Fn(&mut X1::U, &X2::O) + Copy + Send,
    {
        let x2 = self.x2.inspect(h);
        self.with_xap2(x2)
    }

    pub fn filter<H>(self, h: H) -> ParUseRes<U, I, M, E, X1, FilOf<X2, H>, S::ThenBin, R>
    where
        H: Fn(&mut X1::U, &X2::O) -> bool + Copy + Send,
        S::ThenBin: SizePairUseRes,
    {
        let x2 = self.x2.filter(h);
        self.with_xap2(x2)
    }

    pub fn filter_map<Q, H>(
        self,
        h: H,
    ) -> ParUseRes<U, I, M, E, X1, FilMapOf<X2, Q, H>, S::ThenBin, R>
    where
        H: Fn(&mut X1::U, X2::O) -> Option<Q> + Copy + Send,
        S::ThenBin: SizePairUseRes,
    {
        let x2 = self.x2.filter_map(h);
        self.with_xap2(x2)
    }

    pub fn flat_map<V, H>(
        self,
        h: H,
    ) -> ParUseRes<U, I, M, E, X1, FlatMapOf<X2, V, H>, S::ThenMany, R>
    where
        V: IntoIterator,
        H: Fn(&mut X1::U, X2::O) -> V + Copy + Send,
        S::ThenMany: SizePairUseRes,
    {
        let x2 = self.x2.flat_map(h);
        self.with_xap2(x2)
    }

    // compute

    pub fn first(self) -> Result<Option<X2::O>, E>
    where
        X2::O: Send,
        E: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = self.destruct();
        match params.iteration_order {
            IterationOrder::Ordered => exe
                .next(s, params, u, iter, x1, x2)
                .map(|x| x.map(|x| x.val)),
            IterationOrder::Arbitrary => exe.next_any(s, params, u, iter, x1, x2),
        }
    }

    pub fn reduce<F>(self, f: F) -> Result<Option<X2::O>, E>
    where
        F: Fn(&mut X1::U, X2::O, X2::O) -> X2::O + Send + Copy,
        X2::O: Send,
        E: Send,
    {
        let (u, iter, x1, x2, mut exe, s, params) = self.destruct();
        exe.reduce(s, params, u, iter, x1, x2, f)
    }

    pub fn collect_into<C>(self, dst: C) -> Result<C, E>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
        E: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::res_use_col_into(Some(dst), self),
            IterationOrder::Arbitrary => C::res_use_arb_col_into(Some(dst), self),
        }
    }

    pub fn collect<C>(self) -> Result<C, E>
    where
        C: ParCollectInto<X2::O>,
        X2::O: Send,
        E: Send,
    {
        match self.params.iteration_order {
            IterationOrder::Ordered => C::res_use_col_into(None, self),
            IterationOrder::Arbitrary => C::res_use_arb_col_into(None, self),
        }
    }

    // compute - derived

    pub fn for_each<F>(self, f: F)
    where
        F: Fn(&mut X1::U, X2::O) + Send + Copy,
        E: Send,
    {
        let _ = self.map(f).reduce(|_, _, _| {});
    }
}

// transformations

impl<'a, U, O, I, M, E, X1, X2, S, R> ParUseRes<U, I, M, E, X1, X2, S, R>
where
    U: Use,
    O: 'a + Copy,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M, O = &'a O>,
    S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn copied(self) -> ParUseRes<U, I, M, E, X1, MappedOf<X2, FnCopied<'a, U::Item, O>>, S, R> {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseRes::new(u, iter, x1, x2.mapped(FnCopied::new()), exe, params)
    }
}

impl<'a, U, O, I, M, E, X1, X2, S, R> ParUseRes<U, I, M, E, X1, X2, S, R>
where
    U: Use,
    O: 'a + Clone,
    I: ConcurrentIter,
    X1: XapUse<U = U::Item, I = I::Item, O = Result<M, E>>,
    X2: XapUse<U = U::Item, I = M, O = &'a O>,
    S: SizePairUseRes<S1 = X1::Size, S2 = X2::Size>,
    R: ParRunner,
{
    pub fn cloned(self) -> ParUseRes<U, I, M, E, X1, MappedOf<X2, FnCloned<'a, U::Item, O>>, S, R> {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseRes::new(u, iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }
}
