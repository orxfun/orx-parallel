use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, Xap};
use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{UseClone, UseFun};
use crate::option::ParOptionIter;
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParUseOption, Sum};
use crate::{option::ParOptionCore, option_use::ParUseOptionIter};
use core::cmp::Ordering;

pub trait ParOption: Sized + ParOptionCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParOption<
        Item = Self::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParOption<
        Item = Self::Item,
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

    fn using<U, F>(
        self,
        f: F,
    ) -> impl ParUseOption<
        Item = Self::Item,
        Use = U,
        Xap1 = IdUse<Self::Xap1, U>,
        M = Self::M,
        Xap2 = IdUse<Self::Xap2, U>,
        Input = Self::Input,
        Size = Self::Size,
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
        Item = Self::Item,
        Use = U,
        Xap1 = IdUse<Self::Xap1, U>,
        M = Self::M,
        Xap2 = IdUse<Self::Xap2, U>,
        Input = Self::Input,
        Size = Self::Size,
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
        Item = O,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, FnCopied<'a, O>>,
        Input = Self::Input,
        Size = Self::Size,
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
        Item = O,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, FnCloned<'a, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParOption<Item = &'a O>,
        O: Clone + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParOptionIter::new(iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Q,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Self::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = InsOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Self::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilOf<Self::Xap2, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = Q,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FilMapOf<Self::Xap2, Q, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenBin,
    >
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParOption<
        Item = V::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlatMapOf<Self::Xap2, V, H>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
    >
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    fn flatten(
        self,
    ) -> impl ParOption<
        Item = <Self::Item as IntoIterator>::Item,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = FlattenOf<Self::Xap2>,
        Input = Self::Input,
        Size = <Self::Size as SizePair>::ThenMany,
    >
    where
        Self::Item: IntoIterator;

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

    fn all<F>(self, f: F) -> Option<bool>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.map(|x| f(&x))
            .find(|x| *x == false)
            .map(|x| x.is_none())
    }

    fn any<F>(self, f: F) -> Option<bool>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.map(|x| f(&x))
            .find(|x| *x == true)
            .map(|x| x.is_some())
    }

    fn count(self) -> Option<usize> {
        self.map(|_| 1).reduce(|a, b| a + b).map(|x| x.unwrap_or(0))
    }

    fn find<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.filter(&f).first()
    }

    fn for_each<F>(self, f: F) -> Option<()>
    where
        F: Fn(Self::Item) + Send + Copy,
    {
        self.map(f).reduce(|_, _| {}).map(|_| ())
    }

    fn max(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    fn max_by<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<B, F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn min(self) -> Option<Option<Self::Item>>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    fn min_by<F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min_by_key<B, F>(self, f: F) -> Option<Option<Self::Item>>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn sum<S>(self) -> Option<S>
    where
        Self::Item: Sum<S>,
        S: Send,
    {
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
