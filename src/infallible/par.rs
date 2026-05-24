use crate::common_par_traits::ParInfCommon;
use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::xap::FlattenOf;
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, ParIter};
use crate::infallible::{Xap, xap_variants::Id};
use crate::infallible_use::{ParUseIter, UseClone, UseFun, xap_variants::IdUse};
use crate::option::ParOptionIter;
use crate::pool::ParThreadPool;
use crate::result::ParResultIter;
use crate::sizes::Size;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParOption, ParResult, ParUse, Sum,
};
use crate::{infallible::par_core::ParCore, runner::ParRunner};
use core::cmp::Ordering;

pub trait Par: Sized + ParCore + ParInfCommon<CommonItem = Self::Item> {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl Par<Item = Self::Item, Xap = Self::Xap, Input = Self::Input>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl Par<Item = Self::Item, Xap = Self::Xap, Input = Self::Input>;

    fn pool<P: ParThreadPool>(
        self,
        pool: P,
    ) -> impl Par<Item = Self::Item, Xap = Self::Xap, Input = Self::Input>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn into_optional<T>(
        self,
    ) -> impl ParOption<
        Item = T,
        Xap1 = Self::Xap,
        M = T,
        Xap2 = Id<T>,
        Input = Self::Input,
        Size = <<Self::Xap as Xap>::Size as Size>::IntoPair,
    >
    where
        Self::Xap: Xap<O = Option<T>>,
    {
        let (iter, xap, exe, params) = self.destruct();
        let x = ParOptionIter::new(iter, xap, Id::new(), exe, params);
        x
    }

    fn into_fallible<T, E>(
        self,
    ) -> impl ParResult<
        Item = T,
        Error = E,
        Xap1 = Self::Xap,
        M = T,
        Xap2 = Id<T>,
        Input = Self::Input,
        Size = <<Self::Xap as Xap>::Size as Size>::IntoPair,
    >
    where
        Self::Xap: Xap<O = Result<T, E>>,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParResultIter::new(iter, xap, Id::new(), exe, params)
    }

    fn using<U, F>(
        self,
        f: F,
    ) -> impl ParUse<Item = Self::Item, Use = U, Xap = IdUse<Self::Xap, U>, Input = Self::Input>
    where
        F: Fn(usize) -> U + Sync,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UseFun::new(f);
        let xap = IdUse::new(xap);
        ParUseIter::new(using, iter, xap, exe, params)
    }

    fn using_clone<U>(
        self,
        u: U,
    ) -> impl ParUse<Item = Self::Item, Use = U, Xap = IdUse<Self::Xap, U>, Input = Self::Input>
    where
        U: Clone + Send,
    {
        let (iter, xap, exe, params) = self.destruct();
        let using = UseClone::new(u);
        let xap = IdUse::new(xap);
        ParUseIter::new(using, iter, xap, exe, params)
    }

    fn copied<'a, O>(
        self,
    ) -> impl Par<Item = O, Xap = MappedOf<Self::Xap, FnCopied<'a, O>>, Input = Self::Input>
    where
        Self: Par<Item = &'a O>,
        O: Copy + 'a,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParIter::new(iter, xap.mapped(FnCopied::new()), exe, params)
    }

    fn cloned<'a, O>(
        self,
    ) -> impl Par<Item = O, Xap = MappedOf<Self::Xap, FnCloned<'a, O>>, Input = Self::Input>
    where
        Self: Par<Item = &'a O>,
        O: Clone + 'a,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParIter::new(iter, xap.mapped(FnCloned::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl Par<Item = Q, Xap = MapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl Par<Item = Self::Item, Xap = InsOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl Par<Item = Self::Item, Xap = FilOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl Par<Item = Q, Xap = FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl Par<Item = V::Item, Xap = FlatMapOf<Self::Xap, V, H>, Input = Self::Input>
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    fn flatten(
        self,
    ) -> impl Par<
        Item = <Self::Item as IntoIterator>::Item,
        Xap = FlattenOf<Self::Xap>,
        Input = Self::Input,
    >
    where
        Self::Item: IntoIterator;

    // compute

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send;

    fn collect_into<C>(self, dst: &mut C)
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    // compute - derived

    fn all<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.map(|x| f(&x)).find(|x| *x == false).is_none()
    }

    fn any<F>(self, f: F) -> bool
    where
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.map(|x| f(&x)).find(|x| *x == true).is_some()
    }

    fn count(self) -> usize {
        self.map(|_| 1).reduce(|a, b| a + b).unwrap_or(0)
    }

    fn find<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&Self::Item) -> bool + Sync,
    {
        self.filter(&f).first()
    }

    fn for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _| {});
    }

    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::max)
    }

    fn max_by<F>(self, f: F) -> Option<Self::Item>
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

    fn max_by_key<B, F>(self, f: F) -> Option<Self::Item>
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

    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(Ord::min)
    }

    fn min_by<F>(self, f: F) -> Option<Self::Item>
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

    fn min_by_key<B, F>(self, f: F) -> Option<Self::Item>
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

    fn sum<S>(self) -> S
    where
        Self::Item: Sum<S>,
        S: Send,
    {
        self.map(Self::Item::owned)
            .reduce(Self::Item::add)
            .unwrap_or(Self::Item::zero())
    }
}
