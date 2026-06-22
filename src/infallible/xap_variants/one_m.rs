use crate::infallible::XapEnumByInput;
use crate::infallible::fun::Map;
use crate::infallible::fun::MapEnum;
use crate::infallible::xap::{Xap, XapOne};
use crate::sizes::One;

pub struct OneM<X: Xap<Size = One>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Clone for OneM<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Copy for OneM<X, G> {}

impl<X: Xap<Size = One>, G: Map<I = X::O>> OneM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapEnumByInput<Size = One>, G: Map<I = X::O>> XapEnumByInput for OneM<X, G> {
    type Enumerated = OneM<X::Enumerated, MapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapEnum::new(self.g);
        let x = self.x.enumerate();
        OneM::new(x, g)
    }
}

impl<X: Xap<Size = One>, G: Map<I = X::O>> Xap for OneM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = One;

    type Values = [G::O; 1];

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        [self.g.map(self.x.one_value(i))]
    }
}
