use crate::infallible::sizes::Many;
use crate::infallible_using::fun::{FilterMap, FnFil, FnFilMap, FnFlatMap, FnIns, FnMap, Map};
use crate::infallible_using::xap::Xap;
use crate::infallible_using::xap_variants::{ManyM, ManyX};
use core::iter::FusedIterator;

pub struct ManyF<X: Xap<Size = Many>, G: FilterMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: FilterMap<U = X::U, I = X::O>> Clone for ManyF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Many>, G: FilterMap<U = X::U, I = X::O>> Copy for ManyF<X, G> {}

impl<X: Xap<Size = Many>, G: FilterMap<U = X::U, I = X::O>> ManyF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Many>, G: FilterMap<U = X::U, I = X::O>> Xap for ManyF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values = IterManyF<<X::Values as IntoIterator>::IntoIter, G>;

    type U = X::U;

    fn xap(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
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

pub struct IterManyF<I, G>
where
    I: Iterator,
    G: FilterMap<I = I::Item>,
{
    u: *mut G::U,
    i: I,
    g: G,
}

impl<I, G> Iterator for IterManyF<I, G>
where
    I: Iterator,
    G: FilterMap<I = I::Item>,
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
    G: FilterMap<I = I::Item>,
{
}

impl<I, G> DoubleEndedIterator for IterManyF<I, G>
where
    I: DoubleEndedIterator,
    G: FilterMap<I = I::Item>,
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
