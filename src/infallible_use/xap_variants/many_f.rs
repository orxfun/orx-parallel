use crate::infallible_use::XapUse;
use crate::infallible_use::fun::UFilterMap;
use crate::sizes::Many;
use core::iter::FusedIterator;

pub struct UManyF<X: XapUse<Size = Many>, G: UFilterMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Many>, G: UFilterMap<U = X::U, I = X::O>> Clone for UManyF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = Many>, G: UFilterMap<U = X::U, I = X::O>> Copy for UManyF<X, G> {}

impl<X: XapUse<Size = Many>, G: UFilterMap<U = X::U, I = X::O>> UManyF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = Many>, G: UFilterMap<U = X::U, I = X::O>> XapUse for UManyF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values = UIterManyF<<X::Values as IntoIterator>::IntoIter, G>;

    type U = X::U;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let i = self.x.xap_use(u, i).into_iter();
        UIterManyF { u, i, g: self.g }
    }
}

// iter

pub struct UIterManyF<I, G>
where
    I: Iterator,
    G: UFilterMap<I = I::Item>,
{
    u: *mut G::U,
    i: I,
    g: G,
}

impl<I, G> Iterator for UIterManyF<I, G>
where
    I: Iterator,
    G: UFilterMap<I = I::Item>,
{
    type Item = G::O;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
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
        self.i
            .filter_map(|x| self.g.filter_map(unsafe { &mut *self.u }, x))
            .fold(init, f)
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.i
            .filter_map(|x| self.g.filter_map(unsafe { &mut *self.u }, x))
            .count()
    }
}

impl<I, G> FusedIterator for UIterManyF<I, G>
where
    I: FusedIterator,
    G: UFilterMap<I = I::Item>,
{
}

impl<I, G> DoubleEndedIterator for UIterManyF<I, G>
where
    I: DoubleEndedIterator,
    G: UFilterMap<I = I::Item>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
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
