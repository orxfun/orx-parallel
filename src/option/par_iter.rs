use crate::runner::ParRunner;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};

pub trait ParOptIter: Sized {
    type Runner: ParRunner;

    type Size: SizePair;

    type Item;

    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParOptIter<Runner = Q, Size = Self::Size, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParOptIter<Runner = WithDiagnostics<Self::Runner>, Size = Self::Size, Item = Self::Item>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParOptIter<Runner = Self::Runner, Size = Self::Size, Item = Q>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParOptIter<Runner = Self::Runner, Size = Self::Size, Item = Self::Item>
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParOptIter<
        Runner = Self::Runner,
        Size = <Self::Size as SizePair>::ThenBin,
        Item = Self::Item,
    >
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParOptIter<Runner = Self::Runner, Size = <Self::Size as SizePair>::ThenBin, Item = Q>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParOptIter<Runner = Self::Runner, Size = <Self::Size as SizePair>::ThenMany, Item = V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    // compute

    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send;

    fn reduce<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send;

    fn collect_into<C>(self, dst: C) -> Option<C>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    fn collect<C>(self) -> Option<C>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    // compute - derived

    fn for_each<F>(self, f: F) -> Option<()>
    where
        F: Fn(Self::Item) + Send + Copy,
    {
        self.map(f).reduce(|_, _| {}).map(|_| ())
    }
}
