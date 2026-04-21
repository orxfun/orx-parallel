use crate::infallible::{Par, Xap, XapEnumByInput, xap_variants::Id};
use crate::infallible_use::{ParUse, UseClone, UseFun, xap_variants::IdUse};
use crate::option::ParOpt;
use crate::result::ParRes;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::Size;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};
use crate::{infallible::par_iter_core::ParIterCore, runner::ParRunner};
use orx_concurrent_iter::{ConcurrentIter, enumerate::Enumerate};

pub trait ParIter: Sized + ParIterCore {
    // configuration

    fn runner<Q: ParRunner>(self, runner: Q) -> impl ParIter<Runner = Q, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParIter<Runner = WithDiagnostics<Self::Runner>, Item = Self::Item>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    // TODO: return impl ParOptIter
    fn into_optional<T>(
        self,
    ) -> ParOpt<
        <Self as ParIterCore>::Input,
        T,
        <Self as ParIterCore>::Xap,
        Id<T>,
        <<<Self as ParIterCore>::Xap as Xap>::Size as Size>::IntoPair,
        <Self as ParIterCore>::Runner,
    >
    where
        Self::Xap: Xap<O = Option<T>>,
    {
        let (iter, xap, exe, params) = self.destruct();
        let x = ParOpt::new(iter, xap, Id::new(), exe, params);
        x
    }

    fn into_fallible<T, E>(
        self,
    ) -> ParRes<
        Self::Input,
        T,
        E,
        Self::Xap,
        Id<T>,
        <<Self::Xap as Xap>::Size as Size>::IntoPair,
        Self::Runner,
    >
    where
        Self::Xap: Xap<O = Result<T, E>>,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParRes::new(iter, xap, Id::new(), exe, params)
    }

    fn using<U, F>(
        self,
        f: F,
    ) -> ParUse<UseFun<U, F>, Self::Input, IdUse<Self::Xap, U>, Self::Runner>
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UseFun::new(f);
        let xap = IdUse::new(xap);
        ParUse::new(using, iter, xap, exe, params)
    }

    fn using_clone<U>(
        self,
        u: U,
    ) -> ParUse<UseClone<U>, Self::Input, IdUse<Self::Xap, U>, Self::Runner>
    where
        U: Clone + Send,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UseClone::new(u);
        let xap = IdUse::new(xap);
        ParUse::new(using, iter, xap, exe, params)
    }

    fn enumerate(
        self,
    ) -> Par<Enumerate<Self::Input>, <Self::Xap as XapEnumByInput>::Enumerated, Self::Runner>
    where
        Self::Xap: XapEnumByInput,
    {
        let (iter, xap, exe, params) = self.destruct();
        let iter = iter.enumerate();
        let xap = xap.enumerate();
        Par::new(iter, xap, exe, params)
    }

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParIter<Item = Q>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(self, h: H) -> impl ParIter<Item = Self::Item>
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(self, h: H) -> impl ParIter<Item = Self::Item>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> impl ParIter<Item = Q>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> impl ParIter<Item = V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    // compute

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
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
        F: Fn(Self::Item) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _| {});
    }
}
