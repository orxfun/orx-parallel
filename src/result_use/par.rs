use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::{FlattenOf, MappedOf, Use, XapUse};
use crate::result_use::{ParUseResultCore, ParUseResultIter};
use crate::runner::ParRunner;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};

pub trait ParUseResult: Sized + ParUseResultCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseResult<
        Runner = Q,
        Use = Self::Use,
        Size = Self::Size,
        Item = Self::Item,
        Error = Self::Error,
    >;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUseResult<
        Runner = WithDiagnostics<Self::Runner>,
        Use = Self::Use,
        Size = Self::Size,
        Item = Self::Item,
        Error = Self::Error,
    >;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn copied<'a, O>(
        self,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Input = Self::Input,
        Using = Self::Using,
        Use = Self::Use,
        Size = Self::Size,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = MappedOf<Self::Xap2, UFnCopied<'a, Self::Use, O>>,
        Item = O,
        Error = Self::Error,
    >
    where
        Self: ParUseResult<Item = &'a O>,
        O: Copy + 'a,
        Self::Use: 'a,
    {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseResultIter::new(u, iter, x1, x2.mapped(UFnCopied::new()), exe, params)
    }

    fn cloned<'a, O>(
        self,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Input = Self::Input,
        Using = Self::Using,
        Use = Self::Use,
        Size = Self::Size,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = MappedOf<Self::Xap2, UFnCloned<'a, Self::Use, O>>,
        Item = O,
        Error = Self::Error,
    >
    where
        Self: ParUseResult<Item = &'a O>,
        O: Clone + 'a,
        Self::Use: 'a,
    {
        let (u, iter, x1, x2, exe, _, params) = self.destruct();
        ParUseResultIter::new(u, iter, x1, x2.mapped(UFnCloned::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Use = Self::Use,
        Size = Self::Size,
        Item = Q,
        Error = Self::Error,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Use = Self::Use,
        Size = Self::Size,
        Item = Self::Item,
        Error = Self::Error,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, &Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Use = Self::Use,
        Size = <Self::Size as SizePair>::ThenBin,
        Item = Self::Item,
        Error = Self::Error,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, &Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Use = Self::Use,
        Size = <Self::Size as SizePair>::ThenBin,
        Item = Q,
        Error = Self::Error,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Use = Self::Use,
        Size = <Self::Size as SizePair>::ThenMany,
        Item = V::Item,
        Error = Self::Error,
    >
    where
        V: IntoIterator,
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> V + Copy + Send;

    fn flatten(
        self,
    ) -> impl ParUseResult<
        Runner = Self::Runner,
        Use = Self::Use,
        Using = Self::Using,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
        M = Self::M,
        Xap1 = Self::Xap1,
        Xap2 = FlattenOf<Self::Xap2>,
        Item = <Self::Item as IntoIterator>::Item,
        Error = Self::Error,
    >
    where
        Self::Item: IntoIterator;

    // compute

    fn first(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send;

    fn reduce<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        F: Fn(&mut <Self::Using as Use>::Item, Self::Item, Self::Item) -> Self::Item + Send + Copy,
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
        F: Fn(&mut <Self::Using as Use>::Item, Self::Item) + Send + Copy,
        Self::Error: Send,
    {
        self.map(f).reduce(|_, _, _| {}).map(|_| ())
    }
}
