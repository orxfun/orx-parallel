use crate::infallible::xap_variants::Id;
use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{ParUse, ParUseIterCore, XapUse, XapUseEnumByInput};
use crate::option_use::ParUseOpt;
use crate::result_use::ParUseRes;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::Size;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};
use crate::{infallible_use::Use, runner::ParRunner};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_iter::enumerate::Enumerate;

pub trait ParUseIter: Sized + ParUseIterCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseIter<Runner = Q, Use = Self::Use, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUseIter<Runner = WithDiagnostics<Self::Runner>, Use = Self::Use, Item = Self::Item>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn into_optional<T>(
        self,
    ) -> ParUseOpt<
        Self::Use,
        Self::Input,
        T,
        Self::Xap,
        IdUse<Id<T>, <Self::Use as Use>::Item>,
        <<Self::Xap as XapUse>::Size as Size>::IntoPair,
        Self::Runner,
    >
    where
        Self::Xap: XapUse<
                U = <Self::Use as Use>::Item,
                I = <Self::Input as ConcurrentIter>::Item,
                O = Option<T>,
            >,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseOpt::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }

    fn into_fallible<T, E>(
        self,
    ) -> ParUseRes<
        Self::Use,
        Self::Input,
        T,
        E,
        Self::Xap,
        IdUse<Id<T>, <Self::Use as Use>::Item>,
        <<Self::Xap as XapUse>::Size as Size>::IntoPair,
        Self::Runner,
    >
    where
        Self::Xap: XapUse<
                U = <Self::Use as Use>::Item,
                I = <Self::Input as ConcurrentIter>::Item,
                O = Result<T, E>,
            >,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseRes::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }

    fn enumerate(
        self,
    ) -> ParUse<
        Self::Use,
        Enumerate<Self::Input>,
        <Self::Xap as XapUseEnumByInput>::Enumerated,
        Self::Runner,
    >
    where
        Self::Xap: XapUseEnumByInput,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        ParUse::new(u, iter, xap, exe, params)
    }

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParUseIter<Runner = Self::Runner, Use = Self::Use, Item = Q>
    where
        H: Fn(&mut <Self::Use as Use>::Item, Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseIter<Runner = Self::Runner, Use = Self::Use, Item = Self::Item>
    where
        H: Fn(&mut <Self::Use as Use>::Item, &Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseIter<Runner = Self::Runner, Use = Self::Use, Item = Self::Item>
    where
        H: Fn(&mut <Self::Use as Use>::Item, &Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseIter<Runner = Self::Runner, Use = Self::Use, Item = Q>
    where
        H: Fn(&mut <Self::Use as Use>::Item, Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseIter<Runner = Self::Runner, Use = Self::Use, Item = V::Item>
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
