use crate::infallible::XapEnumByInput;
use crate::infallible::fun::Map;
use crate::infallible::fun::MapEnum;
use crate::infallible::xap::{Xap, XapBin};
use crate::sizes::Bin;

/// Zero-or-one xap followed by a map step.
pub struct BinM<X: Xap<Size = Bin>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> Clone for BinM<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> Copy for BinM<X, G> {}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> BinM<X, G> {
    /// Creates an optional mapped xap.
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapEnumByInput<Size = Bin>, G: Map<I = X::O>> XapEnumByInput for BinM<X, G> {
    type Enumerated = BinM<X::Enumerated, MapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapEnum::new(self.g);
        let x = self.x.enumerate();
        BinM::new(x, g)
    }
}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> Xap for BinM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        self.x.bin_value(i).map(|x| self.g.map(x))
    }
}
