use crate::infallible::size::Bin;
use crate::infallible_using::fun::{FnFilMapU, FnFilU, FnFlatMapU, FnInsU, FnMapU, MapU, MapUEnum};
use crate::infallible_using::xap::{Xap, XapBin};
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::xap_variants::{BinF, BinX};

pub struct BinM<X: Xap<Size = Bin>, G: MapU<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: MapU<U = X::U, I = X::O>> Clone for BinM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Bin>, G: MapU<U = X::U, I = X::O>> Copy for BinM<X, G> {}

impl<X: Xap<Size = Bin>, G: MapU<U = X::U, I = X::O>> BinM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapEnumByInput<Size = Bin>, G: MapU<U = X::U, I = X::O>> XapEnumByInput for BinM<X, G> {
    type Enumerated = BinM<X::Enumerated, MapUEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapUEnum::new(self.g);
        let x = self.x.enumerate();
        BinM::new(x, g)
    }
}

impl<X: Xap<Size = Bin>, G: MapU<U = X::U, I = X::O>> Xap for BinM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values<'a>
        = Option<G::O>
    where
        Self: 'a;

    type U = X::U;

    fn xap<'a>(&self, u: &'a mut Self::U, i: Self::I) -> Self::Values<'a>
    where
        Self: 'a,
    {
        self.x.bin_value(u, i).map(|x| self.g.map(u, x))
    }

    // transformations

    type Map<Q, H>
        = BinM<Self, FnMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        BinM::new(self, FnMapU::new(h))
    }

    type Inspect<H>
        = BinM<Self, FnInsU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        BinM::new(self, FnInsU::new(h))
    }

    type Filter<H>
        = BinF<Self, FnFilU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        BinF::new(self, FnFilU::new(h))
    }

    type FilterMap<Q, H>
        = BinF<Self, FnFilMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        BinF::new(self, FnFilMapU::new(h))
    }

    type FlatMap<V, H>
        = BinX<Self, FnFlatMapU<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        BinX::new(self, FnFlatMapU::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = BinM<Self, M>
    where
        M: MapU<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: MapU<U = Self::U, I = Self::O>,
    {
        BinM::new(self, m)
    }
}
