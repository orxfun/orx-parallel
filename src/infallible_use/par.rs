use crate::infallible::xap_variants::Id;
use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{ParUseCore, XapUse};
use crate::option_use::ParUseOptionIter;
use crate::result_use::ParUseResultIter;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::Size;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};
use crate::{infallible_use::Use, runner::ParRunner};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParUse: Sized + ParUseCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUse<Runner = Q, Use = Self::Use, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUse<Runner = WithDiagnostics<Self::Runner>, Use = Self::Use, Item = Self::Item>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn into_optional<T>(
        self,
    ) -> ParUseOptionIter<
        Self::Using,
        Self::Input,
        T,
        Self::Xap,
        IdUse<Id<T>, <Self::Using as Use>::Item>,
        <<Self::Xap as XapUse>::Size as Size>::IntoPair,
        Self::Runner,
    >
    where
        Self::Xap: XapUse<
                U = <Self::Using as Use>::Item,
                I = <Self::Input as ConcurrentIter>::Item,
                O = Option<T>,
            >,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseOptionIter::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }

    fn into_fallible<T, E>(
        self,
    ) -> ParUseResultIter<
        Self::Using,
        Self::Input,
        T,
        E,
        Self::Xap,
        IdUse<Id<T>, <Self::Using as Use>::Item>,
        <<Self::Xap as XapUse>::Size as Size>::IntoPair,
        Self::Runner,
    >
    where
        Self::Xap: XapUse<
                U = <Self::Using as Use>::Item,
                I = <Self::Input as ConcurrentIter>::Item,
                O = Result<T, E>,
            >,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseResultIter::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParUse<Runner = Self::Runner, Use = Self::Use, Item = Q>
    where
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUse<Runner = Self::Runner, Use = Self::Use, Item = Self::Item>
    where
        H: Fn(&mut <Self::Using as Use>::Item, &Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUse<Runner = Self::Runner, Use = Self::Use, Item = Self::Item>
    where
        H: Fn(&mut <Self::Using as Use>::Item, &Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUse<Runner = Self::Runner, Use = Self::Use, Item = Q>
    where
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUse<Runner = Self::Runner, Use = Self::Use, Item = V::Item>
    where
        V: IntoIterator,
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> V + Copy + Send;

    // compute

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(&mut <Self::Using as Use>::Item, Self::Item, Self::Item) -> Self::Item + Send + Copy,
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
        F: Fn(&mut <Self::Using as Use>::Item, Self::Item) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _, _| {});
    }
}
