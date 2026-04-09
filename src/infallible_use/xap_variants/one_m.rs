use crate::infallible_use::fun::{Map, MapEnum};
use crate::infallible_use::{XapOne, XapUse, XapUseEnumByInput};
use crate::sizes::One;

pub struct OneM<X: XapUse<Size = One>, G: Map<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = One>, G: Map<U = X::U, I = X::O>> Clone for OneM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = One>, G: Map<U = X::U, I = X::O>> Copy for OneM<X, G> {}

impl<X: XapUse<Size = One>, G: Map<U = X::U, I = X::O>> OneM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = One>, G: Map<U = X::U, I = X::O>> XapUse for OneM<X, G> {
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

impl<X: XapUseEnumByInput<Size = One>, G: Map<U = X::U, I = X::O>> XapUseEnumByInput
    for OneM<X, G>
{
    type Enumerated = OneM<X::Enumerated, MapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapEnum::new(self.g);
        let x = self.x.enumerate();
        OneM::new(x, g)
    }
}
