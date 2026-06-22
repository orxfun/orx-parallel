use crate::infallible_use::fun::{UMap, UMapEnum};
use crate::infallible_use::{XapUse, XapUseEnumByInput, XapUseOne};
use crate::sizes::One;

pub struct UOneM<X: XapUse<Size = One>, G: UMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = One>, G: UMap<U = X::U, I = X::O>> Clone for UOneM<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: XapUse<Size = One>, G: UMap<U = X::U, I = X::O>> Copy for UOneM<X, G> {}

impl<X: XapUse<Size = One>, G: UMap<U = X::U, I = X::O>> UOneM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = One>, G: UMap<U = X::U, I = X::O>> XapUse for UOneM<X, G> {
    type I = X::I;

    type O = G::O;

    type U = X::U;

    type Size = One;

    type Values = [G::O; 1];

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let u = unsafe { &mut *u };
        let a = self.x.one_value(u, i);
        [self.g.map(u, a)]
    }
}

impl<X: XapUseEnumByInput<Size = One>, G: UMap<U = X::U, I = X::O>> XapUseEnumByInput
    for UOneM<X, G>
{
    type Enumerated = UOneM<X::Enumerated, UMapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = UMapEnum::new(self.g);
        let x = self.x.enumerate();
        UOneM::new(x, g)
    }
}
