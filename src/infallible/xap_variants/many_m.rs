use crate::infallible::XapEnumByInput;
use crate::infallible::fun::{FnFil, FnFilMap};
use crate::infallible::fun::{FnFlatMap, MapEnum};
use crate::infallible::fun::{FnIns, FnMap, Map};
use crate::infallible::sizes::Many;
use crate::infallible::xap::Xap;
use crate::infallible::xap_variants::many_f::ManyF;
use crate::infallible::xap_variants::many_x::ManyX;
use core::iter::FusedIterator;

pub struct ManyM<X: Xap<Size = Many>, G: Map<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: Map<I = X::O>> Clone for ManyM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Many>, G: Map<I = X::O>> Copy for ManyM<X, G> {}

impl<X: Xap<Size = Many>, G: Map<I = X::O>> ManyM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapEnumByInput<Size = Many>, G: Map<I = X::O>> XapEnumByInput for ManyM<X, G> {
    type Enumerated = ManyM<X::Enumerated, MapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapEnum::new(self.g);
        let x = self.x.enumerate();
        ManyM::new(x, g)
    }
}

impl<X: Xap<Size = Many>, G: Map<I = X::O>> Xap for ManyM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values = IterManyM<<X::Values as IntoIterator>::IntoIter, G>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        let i = self.x.xap(i).into_iter();
        IterManyM { i, g: self.g }
    }

    // transformations

    type Inspect<H>
        = ManyM<Self, FnIns<Self::O, H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        ManyM::new(self, FnIns::new(h))
    }

    type Filter<H>
        = ManyF<Self, FnFil<Self::O, H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        ManyF::new(self, FnFil::new(h))
    }

    type FilterMap<Q, H>
        = ManyF<Self, FnFilMap<Self::O, Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        ManyF::new(self, FnFilMap::new(h))
    }

    type FlatMap<V, H>
        = ManyX<Self, FnFlatMap<Self::O, V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        ManyX::new(self, FnFlatMap::new(h))
    }

    // transformations - helper

    type Mapped<M>
        = ManyM<Self, M>
    where
        M: Map<I = Self::O>;

    fn mapped<M>(self, m: M) -> Self::Mapped<M>
    where
        M: Map<I = Self::O>,
    {
        ManyM::new(self, m)
    }
}

// iter

pub struct IterManyM<I, G>
where
    I: Iterator,
    G: Map<I = I::Item>,
{
    i: I,
    g: G,
}

impl<I, G> Iterator for IterManyM<I, G>
where
    I: Iterator,
    G: Map<I = I::Item>,
{
    type Item = G::O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.i.next().map(|x| self.g.map(x))
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
        self.i.map(|x| self.g.map(x)).fold(init, f)
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
    G: Map<I = I::Item>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.i.len()
    }
}

impl<I, G> FusedIterator for IterManyM<I, G>
where
    I: FusedIterator,
    G: Map<I = I::Item>,
{
}

impl<I, G> DoubleEndedIterator for IterManyM<I, G>
where
    I: DoubleEndedIterator,
    G: Map<I = I::Item>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.i.next_back().map(|x| self.g.map(x))
    }
}
