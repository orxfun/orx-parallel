use core::iter::FusedIterator;

use super::fake::Fake;
use crate::infallible::size::{Bin, Many, One, Size};
use crate::infallible_using::fun::MapUEnum;
use crate::infallible_using::xap::{XapBin, XapOne};
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::{fun::MapU, xap::Xap};

pub struct ManyM<X: Xap<Size = Many>, G: MapU<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: MapU<U = X::U, I = X::O>> Clone for ManyM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Many>, G: MapU<U = X::U, I = X::O>> Copy for ManyM<X, G> {}

impl<X: Xap<Size = Many>, G: MapU<U = X::U, I = X::O>> ManyM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

// impl<X: XapEnumByInput<Size = Many>, G: MapU<U = X::U, I = X::O>> XapEnumByInput for ManyM<X, G> {
//     type Enumerated = ManyM<X::Enumerated, MapUEnum<G>>;

//     fn enumerate(self) -> Self::Enumerated {
//         let g = MapUEnum::new(self.g);
//         let x = self.x.enumerate();
//         ManyM::new(x, g)
//     }
// }

// impl<X: Xap<Size = Many>, G: MapU<U = X::U, I = X::O>> Xap for ManyM<X, G> {
//     type I = X::I;

//     type O = G::O;

//     type Size = Many;

//     type Values = IterManyM<<X::Values as IntoIterator>::IntoIter, G>;

//     type U = X::U;

//     fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
//         todo!()
//     }

//     // transformations

//     type Map<Q, H>
//         = Fake<Self::I, Q, Self::U, Self::Size>
//     where
//         H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

//     fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
//     where
//         H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
//     {
//         todo!()
//     }

//     type Inspect<H>
//         = Fake<Self::I, Self::O, Self::U, Self::Size>
//     where
//         H: Fn(&mut Self::U, &Self::O) + Copy + Send;

//     fn inspect<H>(self, h: H) -> Self::Inspect<H>
//     where
//         H: Fn(&mut Self::U, &Self::O) + Copy + Send,
//     {
//         todo!()
//     }

//     type Filter<H>
//         = Fake<Self::I, Self::O, Self::U, <Self::Size as Size>::ThenBin>
//     where
//         H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

//     fn filter<H>(self, h: H) -> Self::Filter<H>
//     where
//         H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
//     {
//         todo!()
//     }

//     type FilterMap<Q, H>
//         = Fake<Self::I, Q, Self::U, <Self::Size as Size>::ThenBin>
//     where
//         H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

//     fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
//     where
//         H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
//     {
//         todo!()
//     }

//     type FlatMap<V, H>
//         = Fake<Self::I, V::Item, Self::U, Many>
//     where
//         V: IntoIterator,
//         H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

//     fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
//     where
//         V: IntoIterator,
//         H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
//     {
//         todo!()
//     }

//     type Mapped<M>
//         = Fake<Self::I, M::O, Self::U, Self::Size>
//     where
//         M: MapU<U = Self::U, I = Self::O>;

//     fn mapped<M>(self, m: M) -> Self::Mapped<M>
//     where
//         M: MapU<U = Self::U, I = Self::O>,
//     {
//         todo!()
//     }
// }

// iter

pub struct IterManyM<'a, I, G>
where
    I: Iterator,
    G: MapU<I = I::Item>,
{
    u: &'a mut G::U,
    i: I,
    g: G,
}

impl<'a, I, G> Iterator for IterManyM<'a, I, G>
where
    I: Iterator,
    G: MapU<I = I::Item>,
{
    type Item = G::O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| self.g.map(self.u, x))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.i.size_hint()
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.i.map(|x| self.g.map(self.u, x)).fold(init, f)
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.i.count()
    }
}

impl<I, G> ExactSizeIterator for IterManyM<'_, I, G>
where
    I: ExactSizeIterator,
    G: MapU<I = I::Item>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.i.len()
    }
}

impl<I, G> FusedIterator for IterManyM<'_, I, G>
where
    I: FusedIterator,
    G: MapU<I = I::Item>,
{
}

impl<I, G> DoubleEndedIterator for IterManyM<'_, I, G>
where
    I: DoubleEndedIterator,
    G: MapU<I = I::Item>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.i.next_back().map(|x| self.g.map(self.u, x))
    }
}
