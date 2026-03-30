use crate::infallible::xap::Xap;
use crate::infallible::{fun::filter_map::FilterMap, size::Many};

pub struct ManyF<X: Xap<Size = Many>, G: FilterMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: FilterMap<I = X::O>> Clone for ManyF<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: Xap<Size = Many>, G: FilterMap<I = X::O>> Copy for ManyF<X, G> {}

impl<X: Xap<Size = Many>, G: FilterMap<I = X::O>> ManyF<X, G> {
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
            match self.i.next() {
                Some(i) => {
                    let y = self.g.filter_map(i);
                    if y.is_some() {
                        return y;
                    }
                }
                None => return None,
            }
        }
    }
}
