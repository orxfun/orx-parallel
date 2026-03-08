use crate::xap::count::{Count, One};
use crate::xap::fun::filter::{FWr, Fs};
use crate::xap::fun::filter_map::{FilMWr, FilterMapFn};
use crate::xap::fun::flat_map::FlaMWr;
use crate::xap::fun::map::{InsWr, MWr, Ms};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fla_map::FlaMap;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct FilMap<X: Xap, G: FilterMapFn<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: FilterMapFn<I = X::O>> FilMap<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: FilterMapFn<I = X::O>> Xap for FilMap<X, G> {
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
        = M<Self, Ms<MWr<Self::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        let h = MWr::new(h);
        M::new(self, Ms::new(h))
    }

    type Inspect<H>
        = M<Self, Ms<InsWr<Self::O, H>>>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        M::new(self, Ms::new(InsWr::new(h)))
    }

    type Filter<H>
        = F<Self, Fs<FWr<Self::O, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F::new(self, Fs::new(FWr::new(h)))
    }

    type FilterMap<Q, H>
        = FilMap<Self, FilMWr<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilMap::new(self, FilMWr::new(h))
    }

    type FlatMap<V, H>
        = FlaMap<Self, FlaMWr<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaMap::new(self, FlaMWr::new(h))
    }
}
