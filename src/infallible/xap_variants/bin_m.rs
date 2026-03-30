use crate::infallible::xap::{Xap, XapBin};
use crate::infallible::{fun::map::Map, size::Bin};

pub struct BinM<X: Xap<Size = Bin>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> Clone for BinM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> Copy for BinM<X, G> {}

impl<X: Xap<Size = Bin>, G: Map<I = X::O>> BinM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
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

    // transformations

    type Map<Q, H>
        = crate::infallible::xap::Fake<Self::I, Q>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        todo!()
    }
}
