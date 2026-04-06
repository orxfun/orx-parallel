use crate::infallible::size::{Bin, Many};
use crate::infallible_using::fun::{FlatMap, FnFilMap, FnFil, FnFlatMap, FnIns, FnMap, Map};
use crate::infallible_using::xap::{Xap, XapBin};
use crate::infallible_using::xap_variants::{ManyF, ManyM, ManyX};
use core::iter::FusedIterator;

pub struct BinX<X: Xap<Size = Bin>, G: FlatMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: FlatMap<U = X::U, I = X::O>> Clone for BinX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Bin>, G: FlatMap<U = X::U, I = X::O>> Copy for BinX<X, G> {}

impl<X: Xap<Size = Bin>, G: FlatMap<U = X::U, I = X::O>> BinX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Bin>, G: FlatMap<U = X::U, I = X::O>> Xap for BinX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = IterBinX<<G::O as IntoIterator>::IntoIter>;

    type U = X::U;

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        let i = self
            .x
            .bin_value(u, i)
            .map(|x| self.g.flat_map(u, x).into_iter());
        IterBinX { i }
    }

    // transformations

    type Map<Q, H>
        = ManyM<Self, FnMap<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        ManyM::new(self, FnMap::new(h))
    }

    type Inspect<H>
        = ManyM<Self, FnIns<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        ManyM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = ManyF<Self, FnFil<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        ManyF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = ManyF<Self, FnFilMap<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        ManyF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = ManyX<Self, FnFlatMap<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        ManyX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = ManyM<Self, M>
    where
        M: Map<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<U = Self::U, I = Self::O>,
    {
        ManyM::new(self, m)
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
