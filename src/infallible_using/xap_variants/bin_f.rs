use crate::infallible_using::fun::FilterMap;
use crate::infallible_using::{XapUse, XapBin};
use crate::sizes::Bin;

pub struct BinF<X: XapUse<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> Clone for BinF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> Copy for BinF<X, G> {}

impl<X: XapUse<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> BinF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = Bin>, G: FilterMap<U = X::U, I = X::O>> XapUse for BinF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    type U = X::U;

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        self.x.bin_value(u, i).and_then(|x| self.g.filter_map(u, x))
    }
}
