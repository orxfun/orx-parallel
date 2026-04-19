#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};
use crate::{infallible_use::Use, runner::ParRunner};

pub trait ParUseIter: Sized {
    type Runner: ParRunner;

    type Item;

    type Use: Use;

    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseIter<Runner = Q, Item = Self::Item, Use = Self::Use>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUseIter<Runner = WithDiagnostics<Self::Runner>, Item = Self::Item, Use = Self::Use>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParUseIter<Runner = Self::Runner, Item = Q, Use = Self::Use>
    where
        H: Fn(&mut <Self::Use as Use>::Item, Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseIter<Runner = Self::Runner, Item = Self::Item, Use = Self::Use>
    where
        H: Fn(&mut <Self::Use as Use>::Item, &Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseIter<Runner = Self::Runner, Item = Self::Item, Use = Self::Use>
    where
        H: Fn(&mut <Self::Use as Use>::Item, &Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseIter<Runner = Self::Runner, Item = Q, Use = Self::Use>
    where
        H: Fn(&mut <Self::Use as Use>::Item, Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseIter<Runner = Self::Runner, Item = V::Item, Use = Self::Use>
    where
        V: IntoIterator,
        H: Fn(&mut <Self::Use as Use>::Item, Self::Item) -> V + Copy + Send;

    // compute

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(&mut <Self::Use as Use>::Item, Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send;

    fn collect_into<C>(self, dst: C) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    // compute - derived

    fn for_each<F>(self, f: F)
    where
        F: Fn(&mut <Self::Use as Use>::Item, Self::Item) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _, _| {});
    }
}
