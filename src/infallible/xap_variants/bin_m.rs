use crate::infallible::fun::FnFlatMap;
use crate::infallible::fun::{FnFil, FnFilMap};
use crate::infallible::fun::{FnIns, FnMap, Map};
use crate::infallible::size::Bin;
use crate::infallible::xap::{Xap, XapBin};
use crate::infallible::xap_variants::bin_f::BinF;
use crate::infallible::xap_variants::bin_x::BinX;

pub struct BinM<X: Xap<Size = Bin>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> Clone for BinM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> Copy for BinM<X, G> {}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> BinM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> Xap for BinM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        self.x.bin_value(i).map(|x| self.g.map(x))
    }

    // transformations

    type Map<Q, H>
        = BinM<Self, FnMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        BinM::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = BinM<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        BinM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = BinF<Self, FnFil<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        BinF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = BinF<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        BinF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = BinX<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        BinX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = BinM<Self, M>
    where
        M: Map<I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<I = Self::O>,
    {
        BinM::new(self, m)
    }
}
