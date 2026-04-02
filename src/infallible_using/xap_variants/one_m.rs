use super::fake::Fake;
use crate::infallible::size::{Many, One, Size};
use crate::infallible_using::fun::MapUEnum;
use crate::infallible_using::xap::XapOne;
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::{fun::MapU, xap::Xap};

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
        = Fake<Self::I, Q, Self::U, Self::Size>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        todo!()
    }

    type Inspect<H>
        = Fake<Self::I, Self::O, Self::U, Self::Size>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        todo!()
    }

    type Filter<H>
        = Fake<Self::I, Self::O, Self::U, <Self::Size as Size>::ThenBin>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        todo!()
    }

    type FilterMap<Q, H>
        = Fake<Self::I, Q, Self::U, <Self::Size as Size>::ThenBin>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        todo!()
    }

    type FlatMap<V, H>
        = Fake<Self::I, V::Item, Self::U, Many>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        todo!()
    }

    type Mapped<M>
        = Fake<Self::I, M::O, Self::U, Self::Size>
    where
        M: MapU<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: MapU<U = Self::U, I = Self::O>,
    {
        todo!()
    }
}
