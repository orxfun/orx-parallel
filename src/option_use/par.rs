use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::{FlattenOf, MappedOf, Use, XapUse};
use crate::option_use::ParUseOptionIter;
use crate::option_use::par_core::ParUseOptionCore;
use crate::runner::ParRunner;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};

pub trait ParUseOption: Sized + ParUseOptionCore {
    // params

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseOption<Runner = Q, Size = Self::Size, Use = Self::Use, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUseOption<
        Runner = WithDiagnostics<Self::Runner>,
        Size = Self::Size,
        Use = Self::Use,
        Item = Self::Item,
    >;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, iteration_order: IterationOrder) -> Self;

    // kind transformations

    fn copied<'a, O>(
        self,
    ) -> impl ParUseOption<
        Runner = Self::Runner,
        Input = Self::Input,
        Using = Self::Using,
        Use = Self::Use,
        Size = Self::Size,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = MappedOf<Self::Xap2, UFnCopied<'a, Self::Use, O>>,
        Item = O,
    >
    where
        Self: ParUseOption<Item = &'a O>,
        O: Copy + 'a,
        Self::Use: 'a,
    {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseOptionIter::new(u, iter, x1, x2.mapped(UFnCopied::new()), exe, params)
    }

    fn cloned<'a, O>(
        self,
    ) -> impl ParUseOption<
        Runner = Self::Runner,
        Input = Self::Input,
        Using = Self::Using,
        Use = Self::Use,
        Size = Self::Size,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = MappedOf<Self::Xap2, UFnCloned<'a, Self::Use, O>>,
        Item = O,
    >
    where
        Self: ParUseOption<Item = &'a O>,
        O: Clone + 'a,
        Self::Use: 'a,
    {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseOptionIter::new(u, iter, x1, x2.mapped(UFnCloned::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseOption<Runner = Self::Runner, Size = Self::Size, Use = Self::Use, Item = Q>
    where
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseOption<Runner = Self::Runner, Size = Self::Size, Use = Self::Use, Item = Self::Item>
    where
        H: Fn(&mut <Self::Using as Use>::Item, &Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Runner = Self::Runner,
        Size = <Self::Size as SizePair>::ThenBin,
        Use = Self::Use,
        Item = Self::Item,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, &Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Runner = Self::Runner,
        Size = <Self::Size as SizePair>::ThenBin,
        Use = Self::Use,
        Item = Q,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseOption<
        Runner = Self::Runner,
        Size = <Self::Size as SizePair>::ThenMany,
        Use = Self::Use,
        Item = V::Item,
    >
    where
        V: IntoIterator,
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> V + Copy + Send;

    fn flatten(
        self,
    ) -> impl ParUseOption<
        Runner = Self::Runner,
        Use = Self::Use,
        Using = Self::Using,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = FlattenOf<Self::Xap2>,
        Item = <Self::Item as IntoIterator>::Item,
    >
    where
        Self::Item: IntoIterator;

    // compute

    fn first(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Send;

    fn reduce<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        F: Fn(&mut <Self::Using as Use>::Item, Self::Item, Self::Item) -> Self::Item + Send + Copy,
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
        F: Fn(&mut <Self::Using as Use>::Item, Self::Item) + Send + Copy,
    {
        self.map(f).reduce(|_, _, _| {}).map(|_| ())
    }
}
