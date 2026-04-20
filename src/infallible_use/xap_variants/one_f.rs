use crate::infallible_use::fun::UFilterMap;
use crate::infallible_use::{XapUse, XapUseOne};
use crate::sizes::{Bin, One};

pub struct UOneF<X: XapUse<Size = One>, G: UFilterMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = One>, G: UFilterMap<U = X::U, I = X::O>> Clone for UOneF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = One>, G: UFilterMap<U = X::U, I = X::O>> Copy for UOneF<X, G> {}

impl<X: XapUse<Size = One>, G: UFilterMap<U = X::U, I = X::O>> UOneF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = One>, G: UFilterMap<U = X::U, I = X::O>> XapUse for UOneF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    type U = X::U;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let u = unsafe { &mut *u };
        let a = self.x.one_value(u, i);
        self.g.filter_map(u, a)
    }
}
