use crate::infallible::fun::filter_map::FilterMap;
use crate::infallible::fun::map::FnMap;
use crate::infallible::size::{Bin, One};
use crate::infallible::xap::{Xap, XapOne};
use crate::infallible::xap_variants::bin_m::BinM;

pub struct OneF<X: Xap<Size = One>, G: FilterMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: FilterMap<I = X::O>> Clone for OneF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = One>, G: FilterMap<I = X::O>> Copy for OneF<X, G> {}

impl<X: Xap<Size = One>, G: FilterMap<I = X::O>> OneF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = One>, G: FilterMap<I = X::O>> Xap for OneF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        self.g.filter_map(self.x.one_value(i))
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
}
