use crate::infallible_use::fun::{Map, MapEnum};
use crate::infallible_use::{XapBin, XapUse, XapUseEnumByInput};
use crate::sizes::Bin;

pub struct BinM<X: XapUse<Size = Bin>, G: Map<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Bin>, G: Map<U = X::U, I = X::O>> Clone for BinM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = Bin>, G: Map<U = X::U, I = X::O>> Copy for BinM<X, G> {}

impl<X: XapUse<Size = Bin>, G: Map<U = X::U, I = X::O>> BinM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUseEnumByInput<Size = Bin>, G: Map<U = X::U, I = X::O>> XapUseEnumByInput
    for BinM<X, G>
{
    type Enumerated = BinM<X::Enumerated, MapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapEnum::new(self.g);
        let x = self.x.enumerate();
        BinM::new(x, g)
    }
}

impl<X: XapUse<Size = Bin>, G: Map<U = X::U, I = X::O>> XapUse for BinM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    type U = X::U;

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        self.x.bin_value(u, i).map(|x| self.g.map(u, x))
    }
}
