use crate::infallible_use::fun::FlatMap;
use crate::infallible_use::{XapUse, XapUseOne};
use crate::sizes::{Many, One};

pub struct OneX<X: XapUse<Size = One>, G: FlatMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = One>, G: FlatMap<U = X::U, I = X::O>> Clone for OneX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = One>, G: FlatMap<U = X::U, I = X::O>> Copy for OneX<X, G> {}

impl<X: XapUse<Size = One>, G: FlatMap<U = X::U, I = X::O>> OneX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = One>, G: FlatMap<U = X::U, I = X::O>> XapUse for OneX<X, G> {
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
