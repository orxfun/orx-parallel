use crate::infallible_use::XapUse;
use crate::infallible_use::fun::UFlatMap;
use crate::sizes::Many;

pub struct UManyX<X: XapUse<Size = Many>, G: UFlatMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Many>, G: UFlatMap<U = X::U, I = X::O>> Clone for UManyX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = Many>, G: UFlatMap<U = X::U, I = X::O>> Copy for UManyX<X, G> {}

impl<X: XapUse<Size = Many>, G: UFlatMap<U = X::U, I = X::O>> UManyX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = Many>, G: UFlatMap<U = X::U, I = X::O>> XapUse for UManyX<X, G> {
    type U = X::U;

    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = UIterManyX<<X::Values as IntoIterator>::IntoIter, G>;

    fn xap_use(&self, u: *mut Self::U, i: Self::I) -> Self::Values {
        let i = self.x.xap_use(u, i).into_iter();
        let (g, inner) = (self.g, None);
        UIterManyX { u, i, g, inner }
    }
}

// iter

pub struct UIterManyX<I, G>
where
    I: Iterator,
    G: UFlatMap<I = I::Item>,
{
    u: *mut G::U,
    i: I,
    g: G,
    inner: Option<<G::O as IntoIterator>::IntoIter>,
}

impl<I, G> Iterator for UIterManyX<I, G>
where
    I: Iterator,
    G: UFlatMap<I = I::Item>,
{
    type Item = <G::O as IntoIterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt;
            }

            match self.i.next() {
                Some(i) => {
                    self.inner = Some(self.g.flat_map(unsafe { &mut *self.u }, i).into_iter())
                }
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
            self.g
                .flat_map(unsafe { &mut *self.u }, i)
                .into_iter()
                .fold(acc, &mut f)
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
            count
                + self
                    .g
                    .flat_map(unsafe { &mut *self.u }, i)
                    .into_iter()
                    .count()
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
