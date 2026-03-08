use crate::xap::count::{Count, One};
use crate::xap::fun::filter::{FnFil, Fs};
use crate::xap::fun::filter_map::{FilterMap, FnFilMap};
use crate::xap::fun::flat_map::FnFlatMap;
use crate::xap::fun::map::{FnIns, FnMap, Ms};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fla_map::FlaMap;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct FilMap<X: Xap, G: FilterMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: FilterMap<I = X::O>> FilMap<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: FilterMap<I = X::O>> Xap for FilMap<X, G> {
    type I = X::I;

    type O = G::O;

    type Count = <X::Count as Count>::ThenZeroOne;

    type Values<'i>
        = <X::Count as Count>::FilterMap<X::Values<'i>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        <X::Count as Count>::filter_map(self.x.xap(i), &self.g)
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
        let h = FnMap::new(h);
        M::new(self, Ms::new(h))
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
        = F<Self, Fs<FnFil<Self::O, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F::new(self, Fs::new(FnFil::new(h)))
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
