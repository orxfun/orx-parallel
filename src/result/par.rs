use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, FlattenOf, InsOf, MapOf, MappedOf, Xap};
use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{UseClone, UseFun};
use crate::result::ParResultIter;
use crate::result::par_core::ParResultCore;
use crate::result_use::ParUseResultIter;
use crate::runner::ParRunner;
use crate::sizes::SizePair;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParUseResult, Sum};
use core::cmp::Ordering;

pub trait ParResult: Sized + ParResultCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParResult<
        Item = Self::Item,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = Self::Xap2,
        Input = Self::Input,
        Size = Self::Size,
    >;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParResult<
        Item = Self::Item,
        Error = Self::Error,
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
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
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
        ParUseResultIter::new(u, iter, x1, x2, exe, params)
    }

    fn using_clone<U>(
        self,
        u: U,
    ) -> impl ParUseResult<
        Item = Self::Item,
        Error = Self::Error,
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
        ParUseResultIter::new(u, iter, x1, x2, exe, params)
    }

    fn copied<'a, O>(
        self,
    ) -> impl ParResult<
        Item = O,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, FnCopied<'a, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParResult<Item = &'a O>,
        O: Copy + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParResultIter::new(iter, x1, x2.mapped(FnCopied::new()), exe, params)
    }

    fn cloned<'a, O>(
        self,
    ) -> impl ParResult<
        Item = O,
        Error = Self::Error,
        Xap1 = Self::Xap1,
        M = Self::M,
        Xap2 = MappedOf<Self::Xap2, FnCloned<'a, O>>,
        Input = Self::Input,
        Size = Self::Size,
    >
    where
        Self: ParResult<Item = &'a O>,
        O: Clone + 'a,
    {
        let (iter, x1, x2, exe, _, params) = self.destruct();
        ParResultIter::new(iter, x1, x2.mapped(FnCloned::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParResult<
        Item = Q,
        Error = Self::Error,
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
    ) -> impl ParResult<
        Item = Self::Item,
        Error = Self::Error,
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
    ) -> impl ParResult<
        Item = Self::Item,
        Error = Self::Error,
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
    ) -> impl ParResult<
        Item = Q,
        Error = Self::Error,
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
    ) -> impl ParResult<
        Item = V::Item,
        Error = Self::Error,
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
    ) -> impl ParResult<
        Item = <Self::Item as IntoIterator>::Item,
        Error = Self::Error,
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
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
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
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.map(|x| f(&x))
            .find(|x| *x == false)
            .map(|x| x.map(|_| false).unwrap_or(true))
    }

    fn any<F>(self, f: F) -> Result<bool, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.map(|x| f(&x))
            .find(|x| *x == true)
            .map(|x| x.is_some())
    }

    fn count(self) -> Result<usize, Self::Error>
    where
        Self::Item: Send,
        Self::Error: Send,
    {
        self.map(|_| 1).reduce(|a, b| a + b).map(|x| x.unwrap_or(0))
    }

    fn find<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
        Self::Error: Send,
    {
        self.filter(&f).first()
    }

    fn for_each<F>(self, f: F) -> Result<(), Self::Error>
    where
        F: Fn(Self::Item) + Send + Copy,
        Self::Error: Send,
    {
        self.map(f).reduce(|_, _| {}).map(|_| ())
    }

    fn max(self) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Ord + Send,
        Self::Error: Send,
    {
        self.reduce(Ord::max)
    }

    fn max_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        Self::Error: Send,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
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
        self.reduce(Ord::min)
    }

    fn min_by<F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        F: Fn(&Self::Item, &Self::Item) -> Ordering + Sync,
        Self::Error: Send,
    {
        let reduce = |x, y| match f(&x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min_by_key<B, F>(self, f: F) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&Self::Item) -> B + Sync,
        Self::Error: Send,
    {
        let reduce = |x, y| match f(&x).cmp(&f(&y)) {
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
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .map(|x| x.unwrap_or(Self::Item::zero()))
    }
}
