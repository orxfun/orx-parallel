use crate::infallible::size::Many;
use crate::infallible_using::fun::{FnFilMapU, FnFilU, FnFlatMapU, FnInsU, FnMapU, MapU, MapUEnum};
use crate::infallible_using::xap::Xap;
use crate::infallible_using::xap_enum::XapEnumByInput;
use crate::infallible_using::xap_variants::{ManyF, ManyX};
use core::iter::FusedIterator;

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

impl<X: XapEnumByInput<Size = Many>, G: MapU<U = X::U, I = X::O>> XapEnumByInput for ManyM<X, G> {
    type Enumerated = ManyM<X::Enumerated, MapUEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapUEnum::new(self.g);
        let x = self.x.enumerate();
        ManyM::new(x, g)
    }
}

impl<X: Xap<Size = Many>, G: MapU<U = X::U, I = X::O>> Xap for ManyM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values<'a>
        = IterManyM<<X::Values<'a> as IntoIterator>::IntoIter, G>
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
        IterManyM {
            u: u_ptr,
            i,
            g: self.g,
        }
    }

    // transformations

    type Map<Q, H>
        = ManyM<Self, FnMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Q + Copy + Send,
    {
        ManyM::new(self, FnMapU::new(h))
    }

    type Inspect<H>
        = ManyM<Self, FnInsU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&mut Self::U, &Self::O) + Copy + Send,
    {
        ManyM::new(self, FnInsU::new(h))
    }

    type Filter<H>
        = ManyF<Self, FnFilU<Self::U, Self::O, H>>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&mut Self::U, &Self::O) -> bool + Copy + Send,
    {
        ManyF::new(self, FnFilU::new(h))
    }

    type FilterMap<Q, H>
        = ManyF<Self, FnFilMapU<Self::U, Self::O, Q, H>>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(&mut Self::U, Self::O) -> Option<Q> + Copy + Send,
    {
        ManyF::new(self, FnFilMapU::new(h))
    }

    type FlatMap<V, H>
        = ManyX<Self, FnFlatMapU<Self::U, Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(&mut Self::U, Self::O) -> V + Copy + Send,
    {
        ManyX::new(self, FnFlatMapU::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = ManyM<Self, M>
    where
        M: MapU<U = Self::U, I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: MapU<U = Self::U, I = Self::O>,
    {
        ManyM::new(self, m)
    }
}

// iter

pub struct IterManyM<I, G>
where
    I: Iterator,
    G: MapU<I = I::Item>,
{
    u: *mut G::U,
    i: I,
    g: G,
}

impl<I, G> Iterator for IterManyM<I, G>
where
    I: Iterator,
    G: MapU<I = I::Item>,
{
    type Item = G::O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: u is either used by i.next or g.map which can never
        // occur at the same time; hence, there exists no race condition
        self.i
            .next()
            .map(|x| self.g.map(unsafe { &mut *self.u }, x))
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
        // SAFETY: u is either used by i.next or g.map which can never
        // occur at the same time; hence, there exists no race condition
        self.i
            .map(|x| self.g.map(unsafe { &mut *self.u }, x))
            .fold(init, f)
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.i.count()
    }
}

impl<I, G> ExactSizeIterator for IterManyM<I, G>
where
    I: ExactSizeIterator,
    G: MapU<I = I::Item>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.i.len()
    }
}

impl<I, G> FusedIterator for IterManyM<I, G>
where
    I: FusedIterator,
    G: MapU<I = I::Item>,
{
}

impl<I, G> DoubleEndedIterator for IterManyM<I, G>
where
    I: DoubleEndedIterator,
    G: MapU<I = I::Item>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        // SAFETY: u is either used by i.next or g.map which can never
        // occur at the same time; hence, there exists no race condition
        self.i
            .next_back()
            .map(|x| self.g.map(unsafe { &mut *self.u }, x))
    }
}
