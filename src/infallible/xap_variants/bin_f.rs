use crate::infallible::fun::FilterMap;
use crate::infallible::xap::{Xap, XapBin};
use crate::sizes::Bin;

/// Zero-or-one xap followed by a filter-map step.
pub struct BinF<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> Clone for BinF<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> Copy for BinF<X, G> {}

impl<X: Xap<Size = Bin>, G: FilterMap<I = X::O>> BinF<X, G> {
    /// Creates an optional filter-map xap.
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
        self.x.bin_value(i).and_then(|x| self.g.filter_map(x))
    }
}
