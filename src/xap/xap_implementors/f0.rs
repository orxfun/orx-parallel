use crate::xap::count::One;
use crate::xap::fun::filter::{FilWrap, FilterQ};
use crate::xap::fun::map::{MapS, MapWrap};
use crate::xap::xap_implementors::fil_m::FilM;
use crate::xap::xap_implementors::fla_m::FlaM;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_implementors::m::M;
use crate::xap::xap_trait::Xap;

pub struct F0<G: FilterQ> {
    g: G,
}

impl<G: FilterQ> F0<G> {
    pub fn new(g: G) -> Self {
        Self { g }
    }
}

impl<G: FilterQ> Xap for F0<G> {
    type I = G::I;

    type O = G::I;

    type Count = One;

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
        = M<Self, MapS<MapWrap<Self::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        M::new(self, MapS::new(MapWrap::new(h)))
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
        = F0<G::Then<FilWrap<G::I, H>>>
    where
        H: Fn(&Self::O) -> bool;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool,
    {
        F0::new(self.g.then(FilWrap::new(h)))
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
