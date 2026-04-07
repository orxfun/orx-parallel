use crate::infallible::XapEnumByInput;
use crate::infallible::fun::{FnFil, FnFilMap};
use crate::infallible::fun::{FnFlatMap, MapEnum};
use crate::infallible::fun::{FnIns, FnMap, Map};
use crate::infallible::sizes::One;
use crate::infallible::xap::{Xap, XapOne};
use crate::infallible::xap_variants::one_f::OneF;
use crate::infallible::xap_variants::one_x::OneX;

pub struct OneM<X: Xap<Size = One>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Clone for OneM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Copy for OneM<X, G> {}

impl<X: Xap<Size = One>, G: Map<I = X::O>> OneM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapEnumByInput<Size = One>, G: Map<I = X::O>> XapEnumByInput for OneM<X, G> {
    type Enumerated = OneM<X::Enumerated, MapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapEnum::new(self.g);
        let x = self.x.enumerate();
        OneM::new(x, g)
    }
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Xap for OneM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = One;

    type Values = [G::O; 1];

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        [self.g.map(self.x.one_value(i))]
    }

    // transformations

    type Inspect<H>
        = OneM<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        OneM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = OneF<Self, FnFil<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        OneF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = OneF<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = OneX<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        OneX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = OneM<Self, M>
    where
        M: Map<I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<I = Self::O>,
    {
        OneM::new(self, m)
    }
}
