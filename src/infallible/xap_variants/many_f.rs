use crate::infallible::fun::FilterMap;
use crate::infallible::xap::Xap;
use crate::sizes::Many;
use core::iter::FusedIterator;

/// Many-valued xap followed by a filter-map step.
pub struct ManyF<X: Xap<Size = Many>, G: FilterMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: FilterMap<I = X::O>> Clone for ManyF<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: Xap<Size = Many>, G: FilterMap<I = X::O>> Copy for ManyF<X, G> {}

impl<X: Xap<Size = Many>, G: FilterMap<I = X::O>> ManyF<X, G> {
    /// Creates a many-valued filter-map xap.
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Many>, G: FilterMap<I = X::O>> Xap for ManyF<X, G> {
    type I = X::I;

    type O = G::O;

    type Size = Many;

    type Values = IterManyF<<X::Values as IntoIterator>::IntoIter, G>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        let i = self.x.xap(i).into_iter();
        IterManyF { i, g: self.g }
    }
}

// iter

/// Iterator returned by a many-valued filter-map xap.
pub struct IterManyF<I, G>
where
    I: Iterator,
    G: FilterMap<I = I::Item>,
{
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
            let i = self.i.next()?;
            if let y @ Some(_) = self.g.filter_map(i) {
                return y;
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
        self.i.filter_map(|x| self.g.filter_map(x)).fold(init, f)
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.i.filter_map(|x| self.g.filter_map(x)).count()
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
            let i = self.i.next_back()?;
            if let y @ Some(_) = self.g.filter_map(i) {
                return y;
            }
        }
    }
}
