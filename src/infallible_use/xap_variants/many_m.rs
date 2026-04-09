use crate::infallible_use::fun::{Map, MapEnum};
use crate::infallible_use::{XapUse, XapUseEnumByInput};
use crate::sizes::Many;
use core::iter::FusedIterator;

pub struct ManyM<X: XapUse<Size = Many>, G: Map<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Many>, G: Map<U = X::U, I = X::O>> Clone for ManyM<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = Many>, G: Map<U = X::U, I = X::O>> Copy for ManyM<X, G> {}

impl<X: XapUse<Size = Many>, G: Map<U = X::U, I = X::O>> ManyM<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUseEnumByInput<Size = Many>, G: Map<U = X::U, I = X::O>> XapUseEnumByInput for ManyM<X, G> {
    type Enumerated = ManyM<X::Enumerated, MapEnum<G>>;

    fn enumerate(self) -> Self::Enumerated {
        let g = MapEnum::new(self.g);
        let x = self.x.enumerate();
        ManyM::new(x, g)
    }
}

impl<X: XapUse<Size = Many>, G: Map<U = X::U, I = X::O>> XapUse for ManyM<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values = IterManyM<<X::Values as IntoIterator>::IntoIter, G>;

    type U = X::U;

    fn xap_use(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        // SAFETY: u is either used by i.next or g.map which can never
        // occur at the same time; hence, there exists no race condition
        let u_ptr = u as *mut Self::U;
        let i = self.x.xap_use(u, i).into_iter();
        IterManyM {
            u: u_ptr,
            i,
            g: self.g,
        }
    }
}

// iter

pub struct IterManyM<I, G>
where
    I: Iterator,
    G: Map<I = I::Item>,
{
    u: *mut G::U,
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
        // SAFETY: u is either used by i.next or g.map which can never
        // occur at the same time; hence, there exists no race condition
        self.i
            .next_back()
            .map(|x| self.g.map(unsafe { &mut *self.u }, x))
    }
}
