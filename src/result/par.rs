use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{MappedOf, Xap};
use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{UseClone, UseFun};
use crate::result::ParResultIter;
use crate::result::par_core::ParResultCore;
use crate::result_use::ParUseResultIter;
use crate::runner::ParRunner;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParUseResult};

pub trait ParResult: Sized + ParResultCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParResult<Runner = Q, Item = Self::Item, Error = Self::Error>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParResult<Runner = WithDiagnostics<Self::Runner>, Item = Self::Item, Error = Self::Error>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn using<U, F>(
        self,
        f: F,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Input = Self::Input,
        Size = Self::Size,
        Use = U,
        Using = UseFun<U, F>,
        M = Self::M,
        Xap1 = IdUse<Self::Xap1, U>,
        Xap2 = IdUse<Self::Xap2, U>,
        Item = Self::Item,
        Error = Self::Error,
    >
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseFun::new(f);
        ParUseResultIter::new(u, iter, x1, x2, exe, params)
    }

    fn using_clone<U>(
        self,
        u: U,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Input = Self::Input,
        Size = Self::Size,
        Use = U,
        Using = UseClone<U>,
        M = Self::M,
        Xap1 = IdUse<Self::Xap1, U>,
        Xap2 = IdUse<Self::Xap2, U>,
        Item = Self::Item,
        Error = Self::Error,
    >
    where
        U: Clone + Send,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseClone::new(u);
        ParUseResultIter::new(u, iter, x1, x2, exe, params)
    }

    fn copied<'a, O>(
        self,
    ) -> ParResultIter<
        Self::Input,
        Self::M,
        Self::Error,
        Self::Xap1,
        MappedOf<Self::Xap2, FnCopied<'a, O>>,
        Self::Size,
        Self::Runner,
    >
    where
        Self: ParResult<Item = &'a O>,
        O: Copy,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParResultIter::new(iter, x1, x2.mapped(FnCopied::new()), exe, params)
    }

    fn cloned<'a, O>(
        self,
    ) -> ParResultIter<
        Self::Input,
        Self::M,
        Self::Error,
        Self::Xap1,
        MappedOf<Self::Xap2, FnCloned<'a, O>>,
        Self::Size,
        Self::Runner,
    >
    where
        Self: ParResult<Item = &'a O>,
        O: Clone,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParResultIter::new(iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParResult<Item = Q, Error = Self::Error>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(self, h: H) -> impl ParResult<Item = Self::Item, Error = Self::Error>
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(self, h: H) -> impl ParResult<Item = Self::Item, Error = Self::Error>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send,
        <Self::Size as SizePair>::ThenBin: SizePair;

    fn filter_map<Q, H>(self, h: H) -> impl ParResult<Item = Q, Error = Self::Error>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send,
        <Self::Size as SizePair>::ThenBin: SizePair;

    fn flat_map<V, H>(self, h: H) -> impl ParResult<Item = V::Item, Error = Self::Error>
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
