use super::fake::Fake;
use crate::infallible::size::{Bin, Many, One, Size};
use crate::infallible_using::fun::{FilterMapU, MapUEnum};
use crate::infallible_using::xap::{XapBin, XapOne};
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::{fun::MapU, xap::Xap};
use core::iter::FusedIterator;

pub struct ManyF<X: Xap<Size = Many>, G: FilterMapU<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: FilterMapU<U = X::U, I = X::O>> Clone for ManyF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Many>, G: FilterMapU<U = X::U, I = X::O>> Copy for ManyF<X, G> {}

impl<X: Xap<Size = Many>, G: FilterMapU<U = X::U, I = X::O>> ManyF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Many>, G: FilterMapU<U = X::U, I = X::O>> Xap for ManyF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values<'a>
        = IterManyF<<X::Values<'a> as IntoIterator>::IntoIter, G>
    where
        Self: 'a;

    type U = X::U;

    fn xap<'a>(&self, u: &'a mut Self::U, i: Self::I) -> Self::Values<'a>
    where
        Self: 'a,
    {
        // SAFETY: u is either used by i.next or g.map which can never
        // occur at the same time; hence, there exists no race condition
        let u_ptr = u as *mut Self::U;
        let i = self.x.xap(u, i).into_iter();
        IterManyF {
            u: u_ptr,
            i,
            g: self.g,
        }
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

pub struct IterManyF<I, G>
where
    I: Iterator,
    G: FilterMapU<I = I::Item>,
{
    u: *mut G::U,
    i: I,
    g: G,
}

impl<I, G> Iterator for IterManyF<I, G>
where
    I: Iterator,
    G: FilterMapU<I = I::Item>,
{
    type Item = G::O;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: u is either used by i.next or g.filter_map which can never
        // occur at the same time; hence, there exists no race condition
        loop {
            match self.i.next() {
                Some(i) => {
                    if let y @ Some(_) = self.g.filter_map(unsafe { &mut *self.u }, i) {
                        return y;
                    }
                }
                None => return None,
            }
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // lb cannot be guaranteed, all might be filtered out
        (0, self.i.size_hint().1)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        // SAFETY: u is either used by i.next or g.filter_map which can never
        // occur at the same time; hence, there exists no race condition
        self.i
            .filter_map(|x| self.g.filter_map(unsafe { &mut *self.u }, x))
            .fold(init, f)
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        // SAFETY: u is either used by i.next or g.filter_map which can never
        // occur at the same time; hence, there exists no race condition
        self.i
            .filter_map(|x| self.g.filter_map(unsafe { &mut *self.u }, x))
            .count()
    }
}

impl<I, G> FusedIterator for IterManyF<I, G>
where
    I: FusedIterator,
    G: FilterMapU<I = I::Item>,
{
}

impl<I, G> DoubleEndedIterator for IterManyF<I, G>
where
    I: DoubleEndedIterator,
    G: FilterMapU<I = I::Item>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // SAFETY: u is either used by i.next or g.filter_map which can never
        // occur at the same time; hence, there exists no race condition
        loop {
            match self.i.next_back() {
                Some(i) => {
                    if let y @ Some(_) = self.g.filter_map(unsafe { &mut *self.u }, i) {
                        return y;
                    }
                }
                None => return None,
            }
        }
    }
}
