use crate::infallible::fun::FilterMap;
use crate::infallible::xap::{Xap, XapOne};
use crate::sizes::{Bin, One};

/// One-to-one xap followed by a filter-map step.
pub struct OneF<X: Xap<Size = One>, G: FilterMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: FilterMap<I = X::O>> Clone for OneF<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: Xap<Size = One>, G: FilterMap<I = X::O>> Copy for OneF<X, G> {}

impl<X: Xap<Size = One>, G: FilterMap<I = X::O>> OneF<X, G> {
    /// Creates a one-to-zero-or-one xap.
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
}
