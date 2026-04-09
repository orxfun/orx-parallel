use crate::infallible::sizes::One;
use crate::infallible_using::fun::{FnFilMap, FnFil, FnFlatMap, FnIns, FnMap, Map, MapEnum};
use crate::infallible_using::xap::{Xap, XapOne};
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::xap_variants::{OneF, OneX};

pub struct OneM<X: Xap<Size = One>, G: Map<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: Map<U = X::U, I = X::O>> Clone for OneM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = One>, G: Map<U = X::U, I = X::O>> Copy for OneM<X, G> {}

impl<X: Xap<Size = One>, G: Map<U = X::U, I = X::O>> OneM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapEnumByInput<Size = One>, G: Map<U = X::U, I = X::O>> XapEnumByInput for OneM<X, G> {
    type Enumerated = OneM<X::Enumerated, MapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapEnum::new(self.g);
        let x = self.x.enumerate();
        OneM::new(x, g)
    }
}

impl<X: Xap<Size = One>, G: Map<U = X::U, I = X::O>> Xap for OneM<X, G> {
    type I = X::I;

    type O = G::O;

    type U = X::U;

    type Size = One;

    type Values = [G::O; 1];

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        let a = self.x.one_value(u, i);
        [self.g.map(u, a)]
    }

    // transformations

    type Map<Q, H>
        = OneM<Self, FnMap<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        OneM::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = OneM<Self, FnIns<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        OneM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = OneF<Self, FnFil<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        OneF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = OneF<Self, FnFilMap<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = OneX<Self, FnFlatMap<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        OneX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = OneM<Self, M>
    where
        M: Map<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<U = Self::U, I = Self::O>,
    {
        OneM::new(self, m)
    }
}
