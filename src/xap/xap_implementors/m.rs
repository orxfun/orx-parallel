use crate::xap::count::Count;
use crate::xap::fun::filter::{FnFil, Fs};
use crate::xap::fun::filter_map::{FnFil2, FnFilMap};
use crate::xap::fun::flat_map::FnFlatMap;
use crate::xap::fun::map::{FnCloned, FnCopied, FnIns, FnMap, MapQueue, Ms};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::flat_map::FlaMap;
use crate::xap::xap_trait::{Xap, XapCloned, XapCopied};

pub struct M<X: Xap, G: MapQueue<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: MapQueue<I = X::O>> M<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: MapQueue<I = X::O>> Xap for M<X, G> {
    type I = X::I;

    type O = G::O;

    type Count = <X::Count as Count>::ThenOne;

    type Values<'i>
        = <X::Count as Count>::Map<X::Values<'i>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        panic!("abc");
        <X::Count as Count>::map(self.x.xap(i), &self.g)
    }

    // transformations

    type Map<Q, H>
        = M<X, G::Then<Q, FnMap<G::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M::new(self.x, self.g.then(FnMap::new(h)))
    }

    type Inspect<H>
        = M<X, G::Then<G::O, FnIns<G::O, H>>>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        M::new(self.x, self.g.then(FnIns::new(h)))
    }

    // type Filter<H>
    //     = F<Self, Fs<FnFil<Self::O, H>>>
    // where
    //     H: Fn(&Self::O) -> bool;

    // fn filter<H>(self, h: H) -> Self::Filter<H>
    // where
    //     H: Fn(&Self::O) -> bool,
    // {
    //     F::new(self, Fs::new(FnFil::new(h)))
    // }

    type Filter<H>
        = FilMap<Self, FnFil2<Self::O, H>>
    // = F<Self, Fs<FnFil<Self::O, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        // F::new(self, Fs::new(FnFil::new(h)))
        FilMap::new(self, FnFil2::new(h))
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

impl<'a, I: 'a + Clone, X: Xap, G: MapQueue<I = X::O, O = &'a I>> XapCloned<'a, I> for M<X, G> {
    type Cloned = M<X, G::Then<I, FnCloned<'a, I>>>;

    fn cloned(self) -> Self::Cloned {
        M::new(self.x, self.g.then(FnCloned::new()))
    }
}

impl<'a, I: 'a + Copy, X: Xap, G: MapQueue<I = X::O, O = &'a I>> XapCopied<'a, I> for M<X, G> {
    type Copied = M<X, G::Then<I, FnCopied<'a, I>>>;

    fn copied(self) -> Self::Copied {
        M::new(self.x, self.g.then(FnCopied::new()))
    }
}
