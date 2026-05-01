use core::cmp::Ordering;

use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::{
    FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, Use, XapUse,
};
use crate::result_use::{ParUseResultCore, ParUseResultIter};
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, Sum};

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
        H: Fn(&mut Self::Use, Self::Item) -> Q + Copy + Send;

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
        H: Fn(&mut Self::Use, &Self::Item) + Copy + Send;

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
        H: Fn(&mut Self::Use, &Self::Item) -> bool + Copy + Send;

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
        H: Fn(&mut Self::Use, Self::Item) -> Option<Q> + Copy + Send;

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
        H: Fn(&mut Self::Use, Self::Item) -> V + Copy + Send;

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
        F: Fn(&mut Self::Use, Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send,
        Self::Error: Send;

    fn collect_into<C>(self, dst: &mut C) -> Result<(), Self::Error>
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

    fn all<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.map(|u, x| f(u, &x))
            .find(|_, x| *x == false)
            .map(|x| x.map(|_| false).unwrap_or(true))
    }

    fn any<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.map(|u, x| f(u, &x))
            .find(|_, x| *x == true)
            .map(|x| x.is_some())
    }

    fn count(self) -> Result<usize, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send,
    {
        self.map(|_, _| 1)
            .reduce(|_, a, b| a + b)
            .map(|x| x.unwrap_or(0))
    }

    fn find<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.filter(&f).first()
    }

    fn for_each<F>(self, f: F) -> Result<(), Self::Error>
    where
        F: Fn(&mut Self::Use, Self::Item) + Send + Copy,
        Self::Error: Send,
    {
        self.map(f).reduce(|_, _, _| {}).map(|_| ())
    }

    fn max(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
    {
        self.reduce(|_, a, b| Ord::max(a, b))
    }

    fn max_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&mut Self::Use, &Self::Item) -> B + Sync,
        Self::Error: Send,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x).cmp(&f(u, &y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn min(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
    {
        self.reduce(|_, a, b| Ord::min(a, b))
    }

    fn min_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&mut Self::Use, &Self::Item) -> B + Sync,
        Self::Error: Send,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x).cmp(&f(u, &y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn sum<S>(self) -> Result<S, Self::Error>
    where
        Self::Item: Sum<S>,
        S: Send,
        Self::Error: Send,
    {
        self.map(|_, x| Self::Item::owned(x))
            .reduce(|_, a, b| Self::Item::add(a, b))
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
