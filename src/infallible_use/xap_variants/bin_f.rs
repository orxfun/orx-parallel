use crate::infallible_use::fun::UFilterMap;
use crate::infallible_use::{XapUse, XapUseBin};
use crate::sizes::Bin;

pub struct UBinF<X: XapUse<Size = Bin>, G: UFilterMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Bin>, G: UFilterMap<U = X::U, I = X::O>> Clone for UBinF<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: XapUse<Size = Bin>, G: UFilterMap<U = X::U, I = X::O>> Copy for UBinF<X, G> {}

impl<X: XapUse<Size = Bin>, G: UFilterMap<U = X::U, I = X::O>> UBinF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = Bin>, G: UFilterMap<U = X::U, I = X::O>> XapUse for UBinF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    type U = X::U;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let u = unsafe { &mut *u };
        self.x.bin_value(u, i).and_then(|x| self.g.filter_map(u, x))
    }
}
