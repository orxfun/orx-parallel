use crate::infallible::fun::flat_map::FlatMap;
use crate::infallible::size::{Many, One};
use crate::infallible::xap::{Xap, XapOne};

pub struct OneX<X: Xap<Size = One>, G: FlatMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = One>, G: FlatMap<I = X::O>> Clone for OneX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = One>, G: FlatMap<I = X::O>> Copy for OneX<X, G> {}

impl<X: Xap<Size = One>, G: FlatMap<I = X::O>> OneX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = One>, G: FlatMap<I = X::O>> Xap for OneX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = G::O;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        self.g.flat_map(self.x.one_value(i))
    }
}
