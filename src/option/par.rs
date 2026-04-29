use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{FlattenOf, MappedOf, Xap};
use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{UseClone, UseFun};
use crate::option::ParOptionIter;
use crate::runner::ParRunner;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParUseOption};
use crate::{option::ParOptionCore, option_use::ParUseOptionIter};

pub trait ParOption: Sized + ParOptionCore {
    // configuration

    fn runner<Q: ParRunner>(self, runner: Q) -> impl ParOption<Runner = Q, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParOption<Runner = WithDiagnostics<Self::Runner>, Item = Self::Item>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn using<U, F>(
        self,
        f: F,
    ) -> impl ParUseOption<
        Runner = Self::Runner,
        Input = Self::Input,
        Use = U,
        Using = UseFun<U, F>,
        Size = Self::Size,
        Xap1 = IdUse<Self::Xap1, U>,
        Xap2 = IdUse<Self::Xap2, U>,
        M = Self::M,
        Item = Self::Item,
    >
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseFun::new(f);
        ParUseOptionIter::new(u, iter, x1, x2, exe, params)
    }

    fn using_clone<U>(
        self,
        u: U,
    ) -> impl ParUseOption<
        Runner = Self::Runner,
        Input = Self::Input,
        Use = U,
        Using = UseClone<U>,
        Size = Self::Size,
        Xap1 = IdUse<Self::Xap1, U>,
        Xap2 = IdUse<Self::Xap2, U>,
        M = Self::M,
        Item = Self::Item,
    >
    where
        U: Clone + Send,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        let x1 = IdUse::<_, U>::new(x1);
        let x2 = IdUse::<_, U>::new(x2);
        let u = UseClone::new(u);
        ParUseOptionIter::new(u, iter, x1, x2, exe, params)
    }

    fn copied<'a, O>(
        self,
    ) -> impl ParOption<
        Runner = Self::Runner,
        Input = Self::Input,
        Size = Self::Size,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = MappedOf<Self::Xap2, FnCopied<'a, O>>,
        Item = O,
    >
    where
        Self: ParOption<Item = &'a O>,
        O: Copy + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParOptionIter::new(iter, x1, x2.mapped(FnCopied::new()), exe, params)
    }

    fn cloned<'a, O>(
        self,
    ) -> impl ParOption<
        Runner = Self::Runner,
        Input = Self::Input,
        Size = Self::Size,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = MappedOf<Self::Xap2, FnCloned<'a, O>>,
        Item = O,
    >
    where
        Self: ParOption<Item = &'a O>,
        O: Clone + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParOptionIter::new(iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(self, h: H) -> impl ParOption<Item = Q>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(self, h: H) -> impl ParOption<Item = Self::Item>
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(self, h: H) -> impl ParOption<Item = Self::Item>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> impl ParOption<Item = Q>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> impl ParOption<Item = V::Item>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    // fn flatten(
    //     self,
    // ) -> impl ParOption<
    //     Runner = Self::Runner,
    //     Input = Self::Input,
    //     Size = Self::Size,
    //     M = Self::M,
    //     Xap1 = Self::Xap1,
    //     Xap2 = FlattenOf<Self::Xap2>,
    //     Item = <Self::Item as IntoIterator>::Item,
    // >
    // where
    //     Self::Item: IntoIterator,
    //     <Self::Size as SizePair>::ThenMany: SizePair;

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
