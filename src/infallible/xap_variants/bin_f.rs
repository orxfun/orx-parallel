use crate::infallible::fun::filter_map::{FilterMap, FnFil, FnFilMap};
use crate::infallible::fun::flat_map::FnFlatMap;
use crate::infallible::fun::map::{FnIns, FnMap, Map};
use crate::infallible::size::Bin;
use crate::infallible::xap::{Xap, XapBin};
use crate::infallible::xap_variants::bin_m::BinM;

pub struct BinF<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> Clone for BinF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> Copy for BinF<X, G> {}

impl<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> BinF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> Xap for BinF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        self.x.bin_value(i).and_then(|x| self.g.filter_map(x))
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
        = crate::infallible::xap::Fake<Self::I, Self::O>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        todo!()
    }

    type Filter<H>
        = crate::infallible::xap::Fake<Self::I, Self::O>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        todo!()
    }

    type FilterMap<Q, H>
        = crate::infallible::xap::Fake<Self::I, Q>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        todo!()
    }

    type FlatMap<V, H>
        = crate::infallible::xap::Fake<Self::I, <V as IntoIterator>::Item>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        todo!()
    }
}
