use crate::infallible::fun::flat_map::FlatMap;
use crate::infallible::size::{Bin, Many};
use crate::infallible::xap::{Xap, XapBin};

pub struct BinX<X: Xap<Size = Bin>, G: FlatMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: FlatMap<I = X::O>> Clone for BinX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Bin>, G: FlatMap<I = X::O>> Copy for BinX<X, G> {}

impl<X: Xap<Size = Bin>, G: FlatMap<I = X::O>> BinX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Bin>, G: FlatMap<I = X::O>> Xap for BinX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = IterBinX<<G::O as IntoIterator>::IntoIter>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        let i = self.x.bin_value(i).map(|x| self.g.flat_map(x).into_iter());
        IterBinX { i }
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

// iter

pub struct IterBinX<I: Iterator> {
    i: Option<I>,
}

impl<I: Iterator> Iterator for IterBinX<I> {
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.i.as_mut().and_then(|x| x.next())
    }
}
