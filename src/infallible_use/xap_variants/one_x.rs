use crate::infallible_use::fun::UFlatMap;
use crate::infallible_use::{XapUse, XapUseOne};
use crate::sizes::{Many, One};

pub struct UOneX<X: XapUse<Size = One>, G: UFlatMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = One>, G: UFlatMap<U = X::U, I = X::O>> Clone for UOneX<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: XapUse<Size = One>, G: UFlatMap<U = X::U, I = X::O>> Copy for UOneX<X, G> {}

impl<X: XapUse<Size = One>, G: UFlatMap<U = X::U, I = X::O>> UOneX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = One>, G: UFlatMap<U = X::U, I = X::O>> XapUse for UOneX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = G::O;

    type U = X::U;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let u = unsafe { &mut *u };
        let a = self.x.one_value(u, i);
        self.g.flat_map(u, a)
    }
}
