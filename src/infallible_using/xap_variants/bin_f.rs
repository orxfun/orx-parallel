use crate::infallible::size::Bin;
use crate::infallible_using::fun::{FilterMap, FnFil, FnFilMap, FnFlatMap, FnIns, FnMap, Map};
use crate::infallible_using::xap::{Xap, XapBin};
use crate::infallible_using::xap_variants::{BinM, BinX};

pub struct BinF<X: Xap<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> Clone for BinF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> Copy for BinF<X, G> {}

impl<X: Xap<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> BinF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> Xap for BinF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    type U = X::U;

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        self.x.bin_value(u, i).and_then(|x| self.g.filter_map(u, x))
    }

    // transformations

    type Map<Q, H>
        = BinM<Self, FnMap<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        BinM::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = BinM<Self, FnIns<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        BinM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = BinF<Self, FnFil<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        BinF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = BinF<Self, FnFilMap<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        BinF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = BinX<Self, FnFlatMap<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        BinX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = BinM<Self, M>
    where
        M: Map<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<U = Self::U, I = Self::O>,
    {
        BinM::new(self, m)
    }
}
