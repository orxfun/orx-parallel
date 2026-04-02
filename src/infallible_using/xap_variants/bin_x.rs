use super::fake::Fake;
use crate::infallible::size::{Bin, Many, One, Size};
use crate::infallible_using::fun::{FlatMapU, MapUEnum};
use crate::infallible_using::xap::{XapBin, XapOne};
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::{fun::MapU, xap::Xap};
use core::iter::FusedIterator;

pub struct BinX<X: Xap<Size = Bin>, G: FlatMapU<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Bin>, G: FlatMapU<U = X::U, I = X::O>> Clone for BinX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Bin>, G: FlatMapU<U = X::U, I = X::O>> Copy for BinX<X, G> {}

impl<X: Xap<Size = Bin>, G: FlatMapU<U = X::U, I = X::O>> BinX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Bin>, G: FlatMapU<U = X::U, I = X::O>> Xap for BinX<X, G> {
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
        = Fake<Self::I, Q, Self::U, Self::Size>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        todo!()
    }

    type Inspect<H>
        = Fake<Self::I, Self::O, Self::U, Self::Size>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        todo!()
    }

    type Filter<H>
        = Fake<Self::I, Self::O, Self::U, <Self::Size as Size>::ThenBin>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        todo!()
    }

    type FilterMap<Q, H>
        = Fake<Self::I, Q, Self::U, <Self::Size as Size>::ThenBin>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        todo!()
    }

    type FlatMap<V, H>
        = Fake<Self::I, V::Item, Self::U, Many>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        todo!()
    }

    type Mapped<M>
        = Fake<Self::I, M::O, Self::U, Self::Size>
    where
        M: MapU<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: MapU<U = Self::U, I = Self::O>,
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
