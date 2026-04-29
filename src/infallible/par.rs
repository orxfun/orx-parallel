use crate::infallible::fun::{FnCloned, FnCopied};
use crate::infallible::{FilMapOf, FilOf, FlatMapOf, InsOf, MapOf, MappedOf, ParIter};
use crate::infallible::{Xap, xap_variants::Id};
use crate::infallible_use::{ParUseIter, UseClone, UseFun, xap_variants::IdUse};
use crate::option::ParOptionIter;
use crate::result::ParResultIter;
#[cfg(feature = "std")]
use crate::runner::WithDiagnostics;
use crate::sizes::Size;
use crate::{ChunkSize, IterationOrder, NumThreads, ParCollectInto, ParOption, ParResult, ParUse};
use crate::{infallible::par_core::ParCore, runner::ParRunner};

pub trait Par: Sized + ParCore {
    // configuration

    fn runner<Q: ParRunner>(
        self,
        runner: Q,
    ) -> impl Par<Runner = Q, Input = Self::Input, Xap = Self::Xap, Item = Self::Item>;

    #[cfg(feature = "std")]
    fn runner_with_diagnostics(
        self,
    ) -> impl Par<
        Runner = WithDiagnostics<Self::Runner>,
        Input = Self::Input,
        Xap = Self::Xap,
        Item = Self::Item,
    >;

    fn num_threads(self, num_threads: impl Into<NumThreads>) -> Self;

    fn chunk_size(self, chunk_size: impl Into<ChunkSize>) -> Self;

    fn iteration_order(self, collect: IterationOrder) -> Self;

    // kind transformations

    fn into_optional<T>(
        self,
    ) -> impl ParOption<
        Runner = Self::Runner,
        Input = Self::Input,
        Size = <<Self::Xap as Xap>::Size as Size>::IntoPair,
        M = T,
        Xap1 = Self::Xap,
        Xap2 = Id<T>,
        Item = T,
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
        Runner = Self::Runner,
        Input = Self::Input,
        Size = <<Self::Xap as Xap>::Size as Size>::IntoPair,
        M = T,
        Xap1 = Self::Xap,
        Xap2 = Id<T>,
        Item = T,
        Error = E,
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
    ) -> impl ParUse<
        Runner = Self::Runner,
        Input = Self::Input,
        Use = U,
        Using = UseFun<U, F>,
        Xap = IdUse<Self::Xap, U>,
        Item = Self::Item,
    >
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
    ) -> impl ParUse<
        Runner = Self::Runner,
        Input = Self::Input,
        Use = U,
        Using = UseClone<U>,
        Xap = IdUse<Self::Xap, U>,
        Item = Self::Item,
    >
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
    ) -> impl Par<
        Runner = Self::Runner,
        Input = Self::Input,
        Xap = MappedOf<Self::Xap, FnCopied<'a, O>>,
        Item = O,
    >
    where
        Self: Par<Item = &'a O>,
        O: Copy + 'a,
    {
        let (iter, xap, exe, params) = self.destruct();
        ParIter::new(iter, xap.mapped(FnCopied::new()), exe, params)
    }

    fn cloned<'a, O>(
        self,
    ) -> impl Par<
        Runner = Self::Runner,
        Input = Self::Input,
        Xap = MappedOf<Self::Xap, FnCloned<'a, O>>,
        Item = O,
    >
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
    ) -> impl Par<Runner = Self::Runner, Input = Self::Input, Xap = MapOf<Self::Xap, Q, H>, Item = Q>
    where
        H: Fn(Self::Item) -> Q + Copy + Send;

    fn inspect<H>(
        self,
        h: H,
    ) -> impl Par<Runner = Self::Runner, Input = Self::Input, Xap = InsOf<Self::Xap, H>, Item = Self::Item>
    where
        H: Fn(&Self::Item) + Copy + Send;

    fn filter<H>(
        self,
        h: H,
    ) -> impl Par<Runner = Self::Runner, Input = Self::Input, Xap = FilOf<Self::Xap, H>, Item = Self::Item>
    where
        H: Fn(&Self::Item) -> bool + Copy + Send;

    fn filter_map<Q, H>(
        self,
        h: H,
    ) -> impl Par<Runner = Self::Runner, Input = Self::Input, Xap = FilMapOf<Self::Xap, Q, H>, Item = Q>
    where
        H: Fn(Self::Item) -> Option<Q> + Copy + Send;

    fn flat_map<V, H>(
        self,
        h: H,
    ) -> impl Par<
        Runner = Self::Runner,
        Input = Self::Input,
        Xap = FlatMapOf<Self::Xap, V, H>,
        Item = V::Item,
    >
    where
        V: IntoIterator,
        H: Fn(Self::Item) -> V + Copy + Send;

    fn flatten(self)
    where
        Self::Item: IntoIterator,
    {
        let map = |e: Self::Item| e.into_iter();
        let x = self.flat_map(map);
        //
    }

    // compute

    fn first(self) -> Option<Self::Item>
    where
        Self::Item: Send;

    fn reduce<F>(self, f: F) -> Option<Self::Item>
    where
        F: Fn(Self::Item, Self::Item) -> Self::Item + Send + Copy,
        Self::Item: Send;

    fn collect_into<C>(self, dst: C) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    fn collect<C>(self) -> C
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send;

    // compute - derived

    fn for_each<F>(self, f: F)
    where
        F: Fn(Self::Item) + Send + Copy,
    {
        let _ = self.map(f).reduce(|_, _| {});
    }
}
