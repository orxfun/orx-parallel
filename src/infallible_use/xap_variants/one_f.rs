use crate::infallible_use::fun::FilterMap;
use crate::infallible_use::{XapUse, XapOne};
use crate::sizes::{Bin, One};

pub struct OneF<X: XapUse<Size = One>, G: FilterMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = One>, G: FilterMap<U = X::U, I = X::O>> Clone for OneF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = One>, G: FilterMap<U = X::U, I = X::O>> Copy for OneF<X, G> {}

impl<X: XapUse<Size = One>, G: FilterMap<U = X::U, I = X::O>> OneF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = One>, G: FilterMap<U = X::U, I = X::O>> XapUse for OneF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    type U = X::U;

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        let a = self.x.one_value(u, i);
        self.g.filter_map(u, a)
    }
}
