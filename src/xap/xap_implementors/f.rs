use crate::xap::count::Count;
use crate::xap::fun::filter::{FilterQueue, FnFil};
use crate::xap::fun::filter_map::FnFilMap;
use crate::xap::fun::flat_map::FnFlatMap;
use crate::xap::fun::map::{FnCloned, FnCopied, FnIns, FnMap, Ms};
use crate::xap::xap_trait::{Xap, XapCloned, XapCopied};
use crate::xap::{FilMap, FlaMap, M};

pub struct F<X: Xap, G: FilterQueue<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: FilterQueue<I = X::O>> F<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: FilterQueue<I = X::O>> Xap for F<X, G> {
    type I = X::I;

    type O = X::O;

    type Count = <X::Count as Count>::ThenZeroOne;

    type Values<'i>
        = <X::Count as Count>::Filter<X::Values<'i>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        <X::Count as Count>::filter(self.x.xap(i), &self.g)
    }

    // transformations

    type Map<Q, H>
        = M<Self, Ms<FnMap<Self::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M::new(self, Ms::new(FnMap::new(h)))
    }

    type Inspect<H>
        = M<Self, Ms<FnIns<Self::O, H>>>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        M::new(self, Ms::new(FnIns::new(h)))
    }

    type Filter<H>
        = F<X, G::Then<FnFil<G::I, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F::new(self.x, self.g.then(FnFil::new(h)))
    }

    type FilterMap<Q, H>
        = FilMap<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilMap::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = FlaMap<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaMap::new(self, FnFlatMap::new(h))
    }
}

impl<'a, I: 'a + Clone, X: Xap<O = &'a I>, G: FilterQueue<I = &'a I>> XapCloned<'a, I> for F<X, G> {
    type Cloned = M<Self, Ms<FnCloned<'a, I>>>;

    fn cloned(self) -> Self::Cloned {
        M::new(self, Ms::new(FnCloned::new()))
    }
}

impl<'a, I: 'a + Copy, X: Xap<O = &'a I>, G: FilterQueue<I = &'a I>> XapCopied<'a, I> for F<X, G> {
    type Copied = M<Self, Ms<FnCopied<'a, I>>>;

    fn copied(self) -> Self::Copied {
        M::new(self, Ms::new(FnCopied::new()))
    }
}
