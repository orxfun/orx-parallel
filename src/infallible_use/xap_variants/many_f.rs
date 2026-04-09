use crate::infallible_use::XapUse;
use crate::infallible_use::fun::FilterMap;
use crate::sizes::Many;
use core::iter::FusedIterator;

pub struct ManyF<X: XapUse<Size = Many>, G: FilterMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Many>, G: FilterMap<U = X::U, I = X::O>> Clone for ManyF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = Many>, G: FilterMap<U = X::U, I = X::O>> Copy for ManyF<X, G> {}

impl<X: XapUse<Size = Many>, G: FilterMap<U = X::U, I = X::O>> ManyF<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = Many>, G: FilterMap<U = X::U, I = X::O>> XapUse for ManyF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values = IterManyF<<X::Values as IntoIterator>::IntoIter, G>;

    type U = X::U;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let i = self.x.xap_use(u, i).into_iter();
        IterManyF { u, i, g: self.g }
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
