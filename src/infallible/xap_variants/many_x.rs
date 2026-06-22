use crate::infallible::fun::FlatMap;
use crate::infallible::xap::Xap;
use crate::sizes::Many;

pub struct ManyX<X: Xap<Size = Many>, G: FlatMap<I = X::O>> {
    x: X,
    g: G,
}

impl<X: Xap<Size = Many>, G: FlatMap<I = X::O>> Clone for ManyX<X, G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<X: Xap<Size = Many>, G: FlatMap<I = X::O>> Copy for ManyX<X, G> {}

impl<X: Xap<Size = Many>, G: FlatMap<I = X::O>> ManyX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: Xap<Size = Many>, G: FlatMap<I = X::O>> Xap for ManyX<X, G> {
    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = IterManyX<<X::Values as IntoIterator>::IntoIter, G>;

    #[inline(always)]
    fn xap(&self, i: Self::I) -> Self::Values {
        let i = self.x.xap(i).into_iter();
        let (g, inner) = (self.g, None);
        IterManyX { i, g, inner }
    }
}

// iter

pub struct IterManyX<I, G>
where
    I: Iterator,
    G: FlatMap<I = I::Item>,
{
    i: I,
    g: G,
    inner: Option<<G::O as IntoIterator>::IntoIter>,
}

impl<I, G> Iterator for IterManyX<I, G>
where
    I: Iterator,
    G: FlatMap<I = I::Item>,
{
    type Item = <G::O as IntoIterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt;
            }

            match self.i.next() {
                Some(i) => self.inner = Some(self.g.flat_map(i).into_iter()),
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

        self.i.fold(acc, |acc, i| {
            self.g.flat_map(i).into_iter().fold(acc, &mut f)
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

        self.i.fold(count, |count, i| {
            count + self.g.flat_map(i).into_iter().count()
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
