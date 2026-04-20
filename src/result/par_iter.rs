use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{UseClone, UseFun};
use crate::result::par_iter_core::ParResIterCore;
use crate::result_use::ParUseRes;
use crate::runner::ParRunner;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};

pub trait ParResIter: Sized + ParResIterCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParResIter<Runner = Q, Item = Self::Item, Error = Self::Error>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParResIter<Runner = WithDiagnostics<Self::Runner>, Item = Self::Item, Error = Self::Error>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn using<U, F>(
        self,
        f: F,
    ) -> ParUseRes<
        UseFun<U, F>,
        Self::Input,
        Self::M,
        Self::Error,
        IdUse<Self::Xap1, U>,
        IdUse<Self::Xap2, U>,
        Self::Size,
        Self::Runner,
    >
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseFun::new(f);
        ParUseRes::new(u, iter, x1, x2, exe, params)
    }

    fn using_clone<U>(
        self,
        u: U,
    ) -> ParUseRes<
        UseClone<U>,
        Self::Input,
        Self::M,
        Self::Error,
        IdUse<Self::Xap1, U>,
        IdUse<Self::Xap2, U>,
        Self::Size,
        Self::Runner,
    >
    where
        U: Clone + Send,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseClone::new(u);
        ParUseRes::new(u, iter, x1, x2, exe, params)
    }

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParResIter<Item = Q, Error = Self::Error>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(self, h: H) -> impl ParResIter<Item = Self::Item, Error = Self::Error>
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(self, h: H) -> impl ParResIter<Item = Self::Item, Error = Self::Error>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send,
        <Self::Size as SizePair>::ThenBin: SizePair;

    fn filter_map<Q, H>(self, h: H) -> impl ParResIter<Item = Q, Error = Self::Error>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send,
        <Self::Size as SizePair>::ThenBin: SizePair;

    fn flat_map<V, H>(self, h: H) -> impl ParResIter<Item = V::Item, Error = Self::Error>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send,
        <Self::Size as SizePair>::ThenMany: SizePair;

    // compute

    fn first(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send;

    fn reduce<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
        Self::Error: Send;

    fn collect_into<C>(self, dst: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Error: Send;

    fn collect<C>(self) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Error: Send;

    // compute - derived

    fn for_each<F>(self, f: F) -> Result<(), Self::Error>
    where
        F: Fn(Self::Item) + Send + Copy,
        Self::Error: Send,
    {
        self.map(f).reduce(|_, _| {}).map(|_| ())
    }
}
