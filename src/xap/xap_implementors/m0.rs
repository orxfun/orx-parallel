use crate::xap::count::One;
use crate::xap::fun::map::{MapQ, MapWrap};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fil_m::FilM;
use crate::xap::xap_implementors::fla_m::FlaM;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_trait::Xap;

pub struct M0<G: MapQ> {
    g: G,
}

impl<G: MapQ> M0<G> {
    pub fn new(g: G) -> Self {
        Self { g }
    }
}

impl<G: MapQ> Xap for M0<G> {
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
        = M0<G::Then<Q, MapWrap<G::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        let h = MapWrap::new(h);
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
        = F<Self, H>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F::new(self, h)
    }

    type FilterMap<Q, H>
        = FilM<Self, Q, H>
    where
        H: Fn(Self::O) -> Option<Q>;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q>,
    {
        FilM::new(self, h)
    }

    type FlatMap<V, H>
        = FlaM<Self, V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V,
    {
        FlaM::new(self, h)
    }
}
