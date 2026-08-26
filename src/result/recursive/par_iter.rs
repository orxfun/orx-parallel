use crate::ParCollectInto;
use crate::infallible::Xap;
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf};
use crate::parameters::{ChunkSize, IterationOrder, NumThreads, Params};
use crate::result::recursive::execution;
use crate::result::recursive::par::ParRecResult;
use crate::result::recursive::par_core::ParRecResultCore;
use crate::runner::{DefaultRunner, ParRunner};
use alloc::vec::Vec;

/// Parallel iterator over `Result` values produced by a recursively expanding computation.
pub struct ParRecResultIter<I, M, E, X1, X2, Ix, Ex, R = DefaultRunner>
where
    I: IntoIterator,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    R: ParRunner,
    Ix: IntoIterator<Item = I::Item>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    iter: I,
    x1: X1,
    x2: X2,
    exe: R,
    params: Params,
    extend: Ex,
}

impl<I, M, E, X1, X2, Ix, Ex, R> ParRecResultIter<I, M, E, X1, X2, Ix, Ex, R>
where
    I: IntoIterator,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    R: ParRunner,
    Ix: IntoIterator<Item = I::Item>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    pub(crate) fn new(iter: I, x1: X1, x2: X2, exe: R, params: Params, extend: Ex) -> Self {
        Self {
            iter,
            x1,
            x2,
            exe,
            params,
            extend,
        }
    }

    fn with_xap2<Y2: Xap<I = M>>(self, x2: Y2) -> ParRecResultIter<I, M, E, X1, Y2, Ix, Ex, R> {
        ParRecResultIter::new(self.iter, self.x1, x2, self.exe, self.params, self.extend)
    }

    fn destruct_x(self) -> (I, X1, X2, R, Params, Ex) {
        (
            self.iter,
            self.x1,
            self.x2,
            self.exe,
            self.params,
            self.extend,
        )
    }
}

impl<I, M, E, X1, X2, Ix, Ex, R> ParRecResultCore for ParRecResultIter<I, M, E, X1, X2, Ix, Ex, R>
where
    I: IntoIterator,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    R: ParRunner,
    Ix: IntoIterator<Item = I::Item>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    type Item = X2::O;

    type Error = E;

    type Runner = R;

    type Input = I;

    type M = M;

    type Xap1 = X1;

    type Xap2 = X2;

    fn destruct(self) -> (Self::Input, Self::Xap1, Self::Xap2, Self::Runner, Params) {
        (self.iter, self.x1, self.x2, self.exe, self.params)
    }
}

impl<I, M, E, X1, X2, Ix, Ex, R> ParRecResult for ParRecResultIter<I, M, E, X1, X2, Ix, Ex, R>
where
    I: IntoIterator,
    M: Send + Sync,
    E: Send + Sync,
    X1: Xap<I = I::Item, O = Result<M, E>>,
    X2: Xap<I = M>,
    R: ParRunner,
    Ix: IntoIterator<Item = I::Item>,
    Ex: Fn(&I::Item) -> Ix + Send + Sync,
{
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParRecResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
    > {
        let (iter, x1, x2, _, params, extend) = self.destruct_x();
        ParRecResultIter::new(iter, x1, x2, runner, params, extend)
    }

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParRecResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
    > {
        let (iter, x1, x2, exe, params, extend) = self.destruct_x();
        ParRecResultIter::new(iter, x1, x2, exe.with_diagnostics(), params, extend)
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

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = Q,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
    >
    where
        H: Fn(X2::O) -> Q + Copy + Send,
    {
        let x2 = self.x2.map(h);
        self.with_xap2(x2)
    }

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
    >
    where
        H: Fn(&X2::O) + Copy + Send,
    {
        let x2 = self.x2.inspect(h);
        self.with_xap2(x2)
    }

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
    >
    where
        H: Fn(&X2::O) -> bool + Copy + Send,
    {
        let x2 = self.x2.filter(h);
        self.with_xap2(x2)
    }

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = Q,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
    >
    where
        H: Fn(X2::O) -> Option<Q> + Copy + Send,
    {
        let x2 = self.x2.filter_map(h);
        self.with_xap2(x2)
    }

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParRecResult<
        Item = V::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
    >
    where
        V: IntoIterator,
        H: Fn(X2::O) -> V + Copy + Send,
    {
        let x2 = self.x2.flat_map(h);
        self.with_xap2(x2)
    }

    fn flatten(
        self,
    ) -> impl ParRecResult<
        Item = <Self::Item as IntoIterator>::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlattenOf<Self::Xap2>,
        Input = Self::Input,
    >
    where
        Self::Item: IntoIterator,
    {
        let x2 = self.x2.flatten();
        self.with_xap2(x2)
    }

    // compute

    fn first(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let (iter, x1, x2, exe, params, extend) = self.destruct_x();

        // TODO: handle ordered, or document it
        match params.iteration_order {
            IterationOrder::Arbitrary | IterationOrder::Ordered => {
                execution::next_any(exe, params, iter, x1, x2, extend)
            }
        }
    }

    fn reduce<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let (iter, x1, x2, exe, params, extend) = self.destruct_x();
        execution::reduce(exe, params, iter, x1, x2, extend, f)
    }

    fn fold<B, Idf, F>(self, init: Idf, f: F) -> Result<Vec<B>, Self::Error>
    where
        B: Send + Sync,
        Idf: Fn() -> B + Sync,
        F: Fn(&mut B, Self::Item) + Copy + Send + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let (iter, x1, x2, exe, params, extend) = self.destruct_x();
        execution::fold(exe, params, iter, x1, x2, extend, init, f)
    }

    fn collect_into<C>(self, dst: &mut C) -> Result<(), Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let (iter, x1, x2, exe, params, extend) = self.destruct_x();

        // TODO: handle ordered, or document it
        match params.iteration_order {
            IterationOrder::Arbitrary | IterationOrder::Ordered => {
                let thread_collections = execution::collect_arb(exe, params, iter, x1, x2, extend)?;
                C::inf_arb_col_into_from_jagged(dst, thread_collections);
                Ok(())
            }
        }
    }

    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send + Sync,
        Self::Error: Send,
        <Self::Input as IntoIterator>::Item: Send + Sync + Clone,
    {
        let mut dst = C::new_empty();
        self.collect_into(&mut dst).map(|_| dst)
    }
}
