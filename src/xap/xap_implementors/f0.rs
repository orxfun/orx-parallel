use crate::xap::count::ZeroOne;
use crate::xap::fun::filter::{FWr, FilterQueue};
use crate::xap::fun::filter_map::FilMWr;
use crate::xap::fun::flat_map::FlaMWr;
use crate::xap::fun::map::{MWr, Ms};
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::fla_map::FlaMap;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::Xap;

pub struct F0<G: FilterQueue> {
    g: G,
}

impl<G: FilterQueue> F0<G> {
    pub fn new(g: G) -> Self {
        Self { g }
    }
}

impl<G: FilterQueue> Xap for F0<G> {
    type I = G::I;

    type O = G::I;

    type Count = ZeroOne;

    type Values<'i>
        = Option<G::I>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        match self.g.filter(&i) {
            true => Some(i),
            false => None,
        }
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
        M::new(self, Ms::new(MWr::new(h)))
    }

    type Inspect<H>
        = Ins<Self, H>
    where
        H: Fn(&Self::O);

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O),
    {
        Ins::new(self, h)
    }

    type Filter<H>
        = F0<G::Then<FWr<G::I, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F0::new(self.g.then(FWr::new(h)))
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
