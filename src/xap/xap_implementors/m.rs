use crate::xap::count::Count;
use crate::xap::fun::filter::{FWr, Fs};
use crate::xap::fun::filter_map::FilMWr;
use crate::xap::fun::map::{MWr, MapQueue};
use crate::xap::xap_implementors::f::F;
use crate::xap::xap_implementors::fil_map::FilMap;
use crate::xap::xap_implementors::fla_map::FlaMap;
use crate::xap::xap_implementors::ins::Ins;
use crate::xap::xap_trait::Xap;

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
        = <Self::Count as Count>::Map<X::Values<'i>, &'i G>
    where
        Self: 'i;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values<'_> {
        <Self::Count as Count>::map(self.x.xap(i), &self.g)
    }

    // transformations

    type Map<Q, H>
        = M<X, G::Then<Q, MWr<G::O, Q, H>>>
    where
        H: Fn(Self::O) -> Q;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q,
    {
        let h = MWr::new(h);
        M::new(self.x, self.g.then(h))
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
