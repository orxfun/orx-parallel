use super::fake::Fake;
use crate::infallible::size::{Bin, Many, One, Size};
use crate::infallible_using::fun::{FlatMapU, MapUEnum};
use crate::infallible_using::xap::{XapBin, XapOne};
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::{fun::MapU, xap::Xap};
use core::iter::FusedIterator;

pub struct ManyX<X: Xap<Size = Many>, G: FlatMapU<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: FlatMapU<U = X::U, I = X::O>> Clone for ManyX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Many>, G: FlatMapU<U = X::U, I = X::O>> Copy for ManyX<X, G> {}

impl<X: Xap<Size = Many>, G: FlatMapU<U = X::U, I = X::O>> ManyX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Many>, G: FlatMapU<U = X::U, I = X::O>> Xap for ManyX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values<'a>
        = IterManyX<<X::Values<'a> as IntoIterator>::IntoIter, G>
    where
        Self: 'a;

    type U = X::U;

    fn xap<'a>(&self, u: &'a mut Self::U, i: Self::I) -> Self::Values<'a>
    where
        Self: 'a,
    {
        // SAFETY: u is either used by i.next or g.flat_map which can never
        // occur at the same time; hence, there exists no race condition
        let u_ptr = u as *mut Self::U;
        let i = self.x.xap(u, i).into_iter();
        let (g, inner, u) = (self.g, None, u_ptr);
        IterManyX { u, i, g, inner }
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

pub struct IterManyX<I, G>
where
    I: Iterator,
    G: FlatMapU<I = I::Item>,
{
    u: *mut G::U,
    i: I,
    g: G,
    inner: Option<<G::O as IntoIterator>::IntoIter>,
}

impl<I, G> Iterator for IterManyX<I, G>
where
    I: Iterator,
    G: FlatMapU<I = I::Item>,
{
    type Item = <G::O as IntoIterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt;
            }

            // SAFETY: u is either used by i.next or g.flat_map which can never
            // occur at the same time; hence, there exists no race condition
            match self.i.next() {
                Some(i) => {
                    self.inner = Some(self.g.flat_map(unsafe { &mut *self.u }, i).into_iter())
                }
                None => return None,
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            Some(inner) => (inner.size_hint().0, None),
            None => (0, None),
        }
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let acc = match self.inner {
            Some(inner) => inner.fold(init, &mut f),
            None => init,
        };

        // SAFETY: u is either used by i.next or g.flat_map which can never
        // occur at the same time; hence, there exists no race condition
        self.i.fold(acc, |acc, i| {
            self.g
                .flat_map(unsafe { &mut *self.u }, i)
                .into_iter()
                .fold(acc, &mut f)
        })
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        let count = match self.inner {
            Some(inner) => inner.count(),
            None => 0,
        };

        // SAFETY: u is either used by i.next or g.flat_map which can never
        // occur at the same time; hence, there exists no race condition
        self.i.fold(count, |count, i| {
            count
                + self
                    .g
                    .flat_map(unsafe { &mut *self.u }, i)
                    .into_iter()
                    .count()
        })
    }
}

#[inline(always)]
fn and_then_or_clear<T, U>(opt: &mut Option<T>, f: impl FnOnce(&mut T) -> Option<U>) -> Option<U> {
    let x = f(opt.as_mut()?);
    if x.is_none() {
        *opt = None;
    }
    x
}
