use crate::infallible::size::{Many, One};
use crate::infallible_using::fun::{FlatMapU, FnFilMapU, FnFilU, FnFlatMapU, FnInsU, FnMapU, MapU};
use crate::infallible_using::xap::{Xap, XapOne};
use crate::infallible_using::xap_variants::{ManyF, ManyM, ManyX};

pub struct OneX<X: Xap<Size = One>, G: FlatMapU<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: FlatMapU<U = X::U, I = X::O>> Clone for OneX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = One>, G: FlatMapU<U = X::U, I = X::O>> Copy for OneX<X, G> {}

impl<X: Xap<Size = One>, G: FlatMapU<U = X::U, I = X::O>> OneX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = One>, G: FlatMapU<U = X::U, I = X::O>> Xap for OneX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = G::O;

    type U = X::U;

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        let a = self.x.one_value(u, i);
        self.g.flat_map(u, a)
    }

    // transformations

    type Map<Q, H>
        = ManyM<Self, FnMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        ManyM::new(self, FnMapU::new(h))
    }

    type Inspect<H>
        = ManyM<Self, FnInsU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        ManyM::new(self, FnInsU::new(h))
    }

    type Filter<H>
        = ManyF<Self, FnFilU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        ManyF::new(self, FnFilU::new(h))
    }

    type FilterMap<Q, H>
        = ManyF<Self, FnFilMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        ManyF::new(self, FnFilMapU::new(h))
    }

    type FlatMap<V, H>
        = ManyX<Self, FnFlatMapU<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        ManyX::new(self, FnFlatMapU::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = ManyM<Self, M>
    where
        M: MapU<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: MapU<U = Self::U, I = Self::O>,
    {
        ManyM::new(self, m)
    }
}
