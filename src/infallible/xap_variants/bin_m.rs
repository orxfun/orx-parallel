use crate::infallible::xap::{Xap, XapBin};
use crate::infallible::{fun::map::Map, size::ZeroOne};

pub struct BinM<X: Xap<Size = ZeroOne>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = ZeroOne>, G: Map<I = X::O>> Clone for BinM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = ZeroOne>, G: Map<I = X::O>> Copy for BinM<X, G> {}

impl<X: Xap<Size = ZeroOne>, G: Map<I = X::O>> BinM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = ZeroOne>, G: Map<I = X::O>> Xap for BinM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = ZeroOne;

    type Values = Option<G::O>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        self.x.opt_value(i).map(|x| self.g.map(x))
    }
}
