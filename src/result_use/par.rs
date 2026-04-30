use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::{
    FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, Use, XapUse,
};
use crate::result_use::{ParUseResultCore, ParUseResultIter};
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto};

pub trait ParUseResult: Sized + ParUseResultCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn copied<'a, O>(
        self,
    ) -> impl ParUseResult<
        Item = O,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, UFnCopied<'a, Self::Use, O>>,
        Input = Self::Input,
        Size = Self::Size,
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
        Item = O,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, UFnCloned<'a, Self::Use, O>>,
        Input = Self::Input,
        Size = Self::Size,
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
        Item = Q,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, &Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, &Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = Q,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUseResult<
        Item = V::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
    >
    where
        V: IntoIterator,
        H: Fn(&mut <Self::Using as Use>::Item, Self::Item) -> V + Copy + Send;

    fn flatten(
        self,
    ) -> impl ParUseResult<
        Item = <Self::Item as IntoIterator>::Item,
        Error = Self::Error,
        Use = Self::Use,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlattenOf<Self::Xap2>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
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
