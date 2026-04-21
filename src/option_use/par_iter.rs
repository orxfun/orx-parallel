use crate::infallible_use::Use;
use crate::option_use::par_iter_core::ParUseOptIterCore;
use crate::runner::ParRunner;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};

pub trait ParUseOptIter: Sized + ParUseOptIterCore {
    // params

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseOptIter<Runner = Q, Size = Self::Size, U = Self::U, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUseOptIter<
        Runner = WithDiagnostics<Self::Runner>,
        Size = Self::Size,
        U = Self::U,
        Item = Self::Item,
    >;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, iteration_order: IterationOrder) -> Self;

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseOptIter<Runner = Self::Runner, Size = Self::Size, U = Self::U, Item = Q>
    where
        H: Fn(&mut <Self::Use as Use>::Item, Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseOptIter<Runner = Self::Runner, Size = Self::Size, U = Self::U, Item = Self::Item>
    where
        H: Fn(&mut <Self::Use as Use>::Item, &Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseOptIter<
        Runner = Self::Runner,
        Size = <Self::Size as SizePair>::ThenBin,
        U = Self::U,
        Item = Self::Item,
    >
    where
        H: Fn(&mut <Self::Use as Use>::Item, &Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseOptIter<
        Runner = Self::Runner,
        Size = <Self::Size as SizePair>::ThenBin,
        U = Self::U,
        Item = Q,
    >
    where
        H: Fn(&mut <Self::Use as Use>::Item, Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseOptIter<
        Runner = Self::Runner,
        Size = <Self::Size as SizePair>::ThenMany,
        U = Self::U,
        Item = V::Item,
    >
    where
        V: IntoIterator,
        H: Fn(&mut <Self::Use as Use>::Item, Self::Item) -> V + Copy + Send;

    // compute

    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send;

    fn reduce<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        F: Fn(&mut <Self::Use as Use>::Item, Self::Item, Self::Item) -> Self::Item + Send + Copy,
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
        F: Fn(&mut <Self::Use as Use>::Item, Self::Item) + Send + Copy,
    {
        self.map(f).reduce(|_, _, _| {}).map(|_| ())
    }
}
