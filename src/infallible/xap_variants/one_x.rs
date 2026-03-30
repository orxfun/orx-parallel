use crate::infallible::fun::filter_map::{FnFil, FnFilMap};
use crate::infallible::fun::flat_map::{FlatMap, FnFlatMap};
use crate::infallible::fun::map::{FnIns, FnMap};
use crate::infallible::size::{Many, One};
use crate::infallible::xap::{Xap, XapOne};
use crate::infallible::xap_variants::many_f::ManyF;
use crate::infallible::xap_variants::many_m::ManyM;
use crate::infallible::xap_variants::many_x::ManyX;

pub struct OneX<X: Xap<Size = One>, G: FlatMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: FlatMap<I = X::O>> Clone for OneX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = One>, G: FlatMap<I = X::O>> Copy for OneX<X, G> {}

impl<X: Xap<Size = One>, G: FlatMap<I = X::O>> OneX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = One>, G: FlatMap<I = X::O>> Xap for OneX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = G::O;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        self.g.flat_map(self.x.one_value(i))
    }

    // transformations

    type Map<Q, H>
        = ManyM<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        ManyM::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = ManyM<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        ManyM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = ManyF<Self, FnFil<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        ManyF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = ManyF<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        ManyF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = ManyX<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        ManyX::new(self, FnFlatMap::new(h))
    }
}
