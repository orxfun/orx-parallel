use crate::infallible_use::fun::{UMap, UMapEnum};
use crate::infallible_use::{XapUse, XapUseEnumByInput};
use crate::sizes::Many;
use core::iter::FusedIterator;

pub struct UManyM<X: XapUse<Size = Many>, G: UMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Many>, G: UMap<U = X::U, I = X::O>> Clone for UManyM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = Many>, G: UMap<U = X::U, I = X::O>> Copy for UManyM<X, G> {}

impl<X: XapUse<Size = Many>, G: UMap<U = X::U, I = X::O>> UManyM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUseEnumByInput<Size = Many>, G: UMap<U = X::U, I = X::O>> XapUseEnumByInput
    for UManyM<X, G>
{
    type Enumerated = UManyM<X::Enumerated, UMapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = UMapEnum::new(self.g);
        let x = self.x.enumerate();
        UManyM::new(x, g)
    }
}

impl<X: XapUse<Size = Many>, G: UMap<U = X::U, I = X::O>> XapUse for UManyM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values = UIterManyM<<X::Values as IntoIterator>::IntoIter, G>;

    type U = X::U;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let i = self.x.xap_use(u, i).into_iter();
        UIterManyM { u, i, g: self.g }
    }
}

// iter

pub struct UIterManyM<I, G>
where
    I: Iterator,
    G: UMap<I = I::Item>,
{
    u: *mut G::U,
    i: I,
    g: G,
}

impl<I, G> Iterator for UIterManyM<I, G>
where
    I: Iterator,
    G: UMap<I = I::Item>,
{
    type Item = G::O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
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

impl<I, G> ExactSizeIterator for UIterManyM<I, G>
where
    I: ExactSizeIterator,
    G: UMap<I = I::Item>,
{
    #[inline(always)]
    fn len(&self) -> usize {
        self.i.len()
    }
}

impl<I, G> FusedIterator for UIterManyM<I, G>
where
    I: FusedIterator,
    G: UMap<I = I::Item>,
{
}

impl<I, G> DoubleEndedIterator for UIterManyM<I, G>
where
    I: DoubleEndedIterator,
    G: UMap<I = I::Item>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.i
            .next_back()
            .map(|x| self.g.map(unsafe { &mut *self.u }, x))
    }
}
