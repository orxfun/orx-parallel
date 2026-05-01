use core::cmp::Ordering;

use crate::infallible::xap_variants::Id;
use crate::infallible_use::fun::{UFnCloned, UFnCopied};
use crate::infallible_use::xap::FlattenOf;
use crate::infallible_use::xap_variants::IdUse;
use crate::infallible_use::{
    FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, ParUseCore, ParUseIter, XapUse,
};
use crate::option_use::ParUseOptionIter;
use crate::result_use::ParUseResultIter;
use crate::runner::ParRunner;
use crate::sizes::Size;
use crate::{
    ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParUseOption, ParUseResult, Sum,
};
use orx_concurrent_iter::ConcurrentIter;

pub trait ParUse: Sized + ParUseCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = Self::Xap, Input = Self::Input>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = Self::Xap, Input = Self::Input>;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn into_optional<T>(
        self,
    ) -> impl ParUseOption<
        Item = T,
        Use = Self::Use,
        Xap1 = Self::Xap,
        M = T,
        Xap2 = IdUse<Id<T>, Self::Use>,
        Input = Self::Input,
        Size = <<Self::Xap as XapUse>::Size as Size>::IntoPair,
    >
    where
        Self::Xap: XapUse<U = Self::Use, I = <Self::Input as ConcurrentIter>::Item, O = Option<T>>,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseOptionIter::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }

    fn into_fallible<T, E>(
        self,
    ) -> impl ParUseResult<
        Item = T,
        Error = E,
        Use = Self::Use,
        Xap1 = Self::Xap,
        M = T,
        Xap2 = IdUse<Id<T>, Self::Use>,
        Input = Self::Input,
        Size = <<Self::Xap as XapUse>::Size as Size>::IntoPair,
    >
    where
        Self::Xap:
            XapUse<U = Self::Use, I = <Self::Input as ConcurrentIter>::Item, O = Result<T, E>>,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseResultIter::new(u, iter, xap, IdUse::new(Id::new()), exe, params)
    }

    fn copied<'a, O>(
        self,
    ) -> impl ParUse<
        Item = O,
        Use = Self::Use,
        Xap = MappedOf<Self::Xap, UFnCopied<'a, Self::Use, O>>,
        Input = Self::Input,
    >
    where
        Self: ParUse<Item = &'a O>,
        O: Copy + 'a,
        Self::Use: 'a,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseIter::new(u, iter, xap.mapped(UFnCopied::new()), exe, params)
    }

    fn cloned<'a, O>(
        self,
    ) -> impl ParUse<
        Item = O,
        Use = Self::Use,
        Xap = MappedOf<Self::Xap, UFnCloned<'a, Self::Use, O>>,
        Input = Self::Input,
    >
    where
        Self: ParUse<Item = &'a O>,
        O: Clone + 'a,
        Self::Use: 'a,
    {
        let (u, iter, xap, exe, params) = self.destruct();
        ParUseIter::new(u, iter, xap.mapped(UFnCloned::new()), exe, params)
    }

    // transformations

    fn map<Q, H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Q, Use = Self::Use, Xap = MapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = InsOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, &Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Self::Item, Use = Self::Use, Xap = FilOf<Self::Xap, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, &Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl ParUse<Item = Q, Use = Self::Use, Xap = FilMapOf<Self::Xap, Q, H>, Input = Self::Input>
    where
        H: Fn(&mut Self::Use, Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl ParUse<
        Item = V::Item,
        Use = Self::Use,
        Xap = FlatMapOf<Self::Xap, V, H>,
        Input = Self::Input,
    >
    where
        V: IntoIterator,
        H: Fn(&mut Self::Use, Self::Item) -> V + Copy + Send;

    fn flatten(
        self,
    ) -> impl ParUse<
        Item = <Self::Item as IntoIterator>::Item,
        Use = Self::Use,
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
        F: Fn(&mut Self::Use, Self::Item, Self::Item) -> Self::Item + Send + Copy,
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
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.map(|u, x| f(u, &x)).find(|_, x| *x == false).is_none()
    }

    fn any<F>(self, f: F) -> bool
    where
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.map(|u, x| f(u, &x)).find(|_, x| *x == true).is_some()
    }

    fn count(self) -> usize {
        self.map(|_, _| 1).reduce(|_, a, b| a + b).unwrap_or(0)
    }

    fn find<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item) -> bool + Sync,
    {
        self.filter(&f).first()
    }

    fn for_each<F>(self, f: F)
    where
        F: Fn(&mut Self::Use, Self::Item) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _, _| {});
    }

    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::max(a, b))
    }

    fn max_by<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x, &y) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn max_by_key<B, F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&mut Self::Use, &Self::Item) -> B + Sync,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x).cmp(&f(u, &y)) {
            Ordering::Greater | Ordering::Equal => x,
            Ordering::Less => y,
        };
        self.reduce(reduce)
    }

    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord + Send,
    {
        self.reduce(|_, a, b| Ord::min(a, b))
    }

    fn min_by<F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        F: Fn(&mut Self::Use, &Self::Item, &Self::Item) -> Ordering + Sync,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x, &y) {
            Ordering::Less | Ordering::Equal => x,
            Ordering::Greater => y,
        };
        self.reduce(reduce)
    }

    fn min_by_key<B, F>(self, f: F) -> Option<Self::Item>
    where
        Self::Item: Send,
        B: Ord,
        F: Fn(&mut Self::Use, &Self::Item) -> B + Sync,
    {
        let reduce = |u: &mut Self::Use, x, y| match f(u, &x).cmp(&f(u, &y)) {
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
        self.map(|_, x| Self::Item::owned(x))
            .reduce(|_, a, b| Self::Item::add(a, b))
            .unwrap_or(Self::Item::zero())
    }
}
