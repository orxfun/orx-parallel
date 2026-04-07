use crate::infallible::fun::FlatMap;
use crate::infallible::sizes::{Bin, Many};
use crate::infallible::xap::{Xap, XapBin};
use core::iter::FusedIterator;

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

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.i {
            Some(i) => i.size_hint(),
            None => (0, Some(0)),
        }
    }

    #[inline(always)]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self.i {
            Some(i) => i.fold(init, f),
            None => init,
        }
    }

    #[inline(always)]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self.i {
            Some(i) => i.count(),
            None => 0,
        }
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for IterBinX<I> {
    #[inline(always)]
    fn len(&self) -> usize {
        match &self.i {
            Some(i) => i.len(),
            None => 0,
        }
    }
}

impl<I: FusedIterator> FusedIterator for IterBinX<I> {}

impl<I: DoubleEndedIterator> DoubleEndedIterator for IterBinX<I> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.i.as_mut().and_then(|x| x.next_back())
    }
}
