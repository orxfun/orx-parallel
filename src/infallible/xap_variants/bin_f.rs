use crate::infallible::xap::{Xap, XapBin};
use crate::infallible::{fun::filter_map::FilterMap, size::Bin};

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
        self.x.opt_value(i).and_then(|x| self.g.filter_map(x))
    }
}
