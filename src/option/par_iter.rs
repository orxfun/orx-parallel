#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};
use crate::{option::SizePairOpt, runner::ParRunner};

pub trait ParOptIter: Sized {
    type Runner: ParRunner;

    type Size: SizePairOpt;

    type Item;

    // configuration

    fn runner<Q: ParRunner>(self, runner: Q) -> impl ParOptIter<Runner = Q, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParOptIter<Runner = WithDiagnostics<Self::Runner>, Item = Self::Item>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParOptIter<Runner = Self::Runner, Item = Q>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(self, h: H) -> impl ParOptIter<Runner = Self::Runner, Item = Self::Item>
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(self, h: H) -> impl ParOptIter<Runner = Self::Runner, Item = Self::Item>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send,
        <Self::Size as SizePair>::ThenBin: SizePairOpt;

    fn filter_map<Q, H>(self, h: H) -> impl ParOptIter<Runner = Self::Runner, Item = Q>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send,
        <Self::Size as SizePair>::ThenBin: SizePairOpt;

    fn flat_map<V, H>(self, h: H) -> impl ParOptIter<Runner = Self::Runner, Item = V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send,
        <Self::Size as SizePair>::ThenMany: SizePairOpt;

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
