use crate::infallible::size::One;
use crate::infallible_using::fun::{FnFilMapU, FnFilU, FnFlatMapU, FnInsU, FnMapU, MapU, MapUEnum};
use crate::infallible_using::xap::{Xap, XapOne};
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::xap_variants::{OneF, OneX};

pub struct OneM<X: Xap<Size = One>, G: MapU<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: MapU<U = X::U, I = X::O>> Clone for OneM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = One>, G: MapU<U = X::U, I = X::O>> Copy for OneM<X, G> {}

impl<X: Xap<Size = One>, G: MapU<U = X::U, I = X::O>> OneM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapEnumByInput<Size = One>, G: MapU<U = X::U, I = X::O>> XapEnumByInput for OneM<X, G> {
    type Enumerated = OneM<X::Enumerated, MapUEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapUEnum::new(self.g);
        let x = self.x.enumerate();
        OneM::new(x, g)
    }
}

impl<X: Xap<Size = One>, G: MapU<U = X::U, I = X::O>> Xap for OneM<X, G> {
    type I = X::I;

    type O = G::O;

    type U = X::U;

    type Size = One;

    type Values<'a>
        = [G::O; 1]
    where
        Self: 'a;

    fn xap<'a>(&self, u: &'a mut Self::U, i: Self::I) -> Self::Values<'a>
    where
        Self: 'a,
    {
        let a = self.x.one_value(u, i);
        [self.g.map(u, a)]
    }

    // transformations

    type Map<Q, H>
        = OneM<Self, FnMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        OneM::new(self, FnMapU::new(h))
    }

    type Inspect<H>
        = OneM<Self, FnInsU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        OneM::new(self, FnInsU::new(h))
    }

    type Filter<H>
        = OneF<Self, FnFilU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        OneF::new(self, FnFilU::new(h))
    }

    type FilterMap<Q, H>
        = OneF<Self, FnFilMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        OneF::new(self, FnFilMapU::new(h))
    }

    type FlatMap<V, H>
        = OneX<Self, FnFlatMapU<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        OneX::new(self, FnFlatMapU::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = OneM<Self, M>
    where
        M: MapU<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: MapU<U = Self::U, I = Self::O>,
    {
        OneM::new(self, m)
    }
}
