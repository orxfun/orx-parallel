use crate::xap::count::One;
use crate::xap::fun::filter::{FWr, Fs};
use crate::xap::fun::map::{MapQueue, MWr};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::fla_map::FlaMap;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_trait::Xap;

pub struct M0<G: MapQueue> {
    g: G,
}

impl<G: MapQueue> M0<G> {
    pub fn new(g: G) -> Self {
        Self { g }
    }
}

impl<G: MapQueue> Xap for M0<G> {
    type I = G::I;

    type O = G::O;

    type Count = One;

    type Values<'i>
        = [G::O; 1]
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        [self.g.map(i)]
    }

    // transformations

    type Map<Q, H>
        = M0<G::Then<Q, MWr<G::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        let h = MWr::new(h);
        M0::new(self.g.then(h))
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
        = FilMap<Self, Q, H>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilMap::new(self, h)
    }

    type FlatMap<V, H>
        = FlaMap<Self, V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaMap::new(self, h)
    }
}
