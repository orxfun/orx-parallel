use crate::xap::M;
use crate::xap::count::{Count, One};
use crate::xap::fun::map::{MapQ, MapWrap};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fil_m::FilM;
use crate::xap::xap_implementors::fla_m::FlaM;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_trait::{IterOf, Xap};

pub struct Mb<X: Xap, G: MapQ<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap, G: MapQ<I = X::O>> Mb<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap, G: MapQ<I = X::O>> Xap for Mb<X, G> {
    type I = X::I;

    type O = G::O;

    type Count = <X::Count as Count>::ThenOne;

    type Values<'i>
        // = MapI<IterOf<'i, X>, &'i G>
        = [Self::O; 1]
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        let x = self.x.xap(i).into_iter().next().unwrap();
        [self.g.map(x)]
        // MapI::new(self.x.xap(i).into_iter(), &self.g)
    }

    // transformations

    type Map<Q, H>
        = Mb<X, G::Then<Q, MapWrap<G::O, Q, H>>>
    // = M<Self, Q, H>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        // M::new(self, h)
        let h = MapWrap::new(h);
        Mb::new(self.x, self.g.then(h))
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
