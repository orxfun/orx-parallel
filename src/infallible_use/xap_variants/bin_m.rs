use crate::infallible_use::fun::{UMap, UMapEnum};
use crate::infallible_use::{XapUse, XapUseBin, XapUseEnumByInput};
use crate::sizes::Bin;

pub struct UBinM<X: XapUse<Size = Bin>, G: UMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Bin>, G: UMap<U = X::U, I = X::O>> Clone for UBinM<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: XapUse<Size = Bin>, G: UMap<U = X::U, I = X::O>> Copy for UBinM<X, G> {}

impl<X: XapUse<Size = Bin>, G: UMap<U = X::U, I = X::O>> UBinM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUseEnumByInput<Size = Bin>, G: UMap<U = X::U, I = X::O>> XapUseEnumByInput
    for UBinM<X, G>
{
    type Enumerated = UBinM<X::Enumerated, UMapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = UMapEnum::new(self.g);
        let x = self.x.enumerate();
        UBinM::new(x, g)
    }
}

impl<X: XapUse<Size = Bin>, G: UMap<U = X::U, I = X::O>> XapUse for UBinM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Bin;

    type Values = Option<G::O>;

    type U = X::U;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let u = unsafe { &mut *u };
        self.x.bin_value(u, i).map(|x| self.g.map(u, x))
    }
}
