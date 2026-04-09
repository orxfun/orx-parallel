use crate::infallible_use::XapUse;
use crate::infallible_use::fun::FlatMap;
use crate::sizes::Many;

pub struct ManyX<X: XapUse<Size = Many>, G: FlatMap<U = X::U, I = X::O>> {
    x: X,
    g: G,
}

impl<X: XapUse<Size = Many>, G: FlatMap<U = X::U, I = X::O>> Clone for ManyX<X, G> {
    fn clone(&self) -> Self {
        Self::new(self.x, self.g)
    }
}

impl<X: XapUse<Size = Many>, G: FlatMap<U = X::U, I = X::O>> Copy for ManyX<X, G> {}

impl<X: XapUse<Size = Many>, G: FlatMap<U = X::U, I = X::O>> ManyX<X, G> {
    pub fn new(x: X, g: G) -> Self {
        Self { x, g }
    }
}

impl<X: XapUse<Size = Many>, G: FlatMap<U = X::U, I = X::O>> XapUse for ManyX<X, G> {
    type U = X::U;

    type I = X::I;

    type O = <G::O as IntoIterator>::Item;

    type Size = Many;

    type Values = IterManyX<<X::Values as IntoIterator>::IntoIter, G>;

    fn xap_use(&self, u: &mut Self::U, i: Self::I) -> Self::Values {
        // SAFETY: u is either used by i.next or g.flat_map which can never
        // occur at the same time; hence, there exists no race condition
        let u_ptr = u as *mut Self::U;
        let i = self.x.xap_use(u, i).into_iter();
        let (g, inner, u) = (self.g, None, u_ptr);
        IterManyX { u, i, g, inner }
    }
}

// iter

pub struct IterManyX<I, G>
where
    I: Iterator,
    G: FlatMap<I = I::Item>,
{
    u: *mut G::U,
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

            // SAFETY: u is either used by i.next or g.flat_map which can never
            // occur at the same time; hence, there exists no race condition
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

        // SAFETY: u is either used by i.next or g.flat_map which can never
        // occur at the same time; hence, there exists no race condition
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

        // SAFETY: u is either used by i.next or g.flat_map which can never
        // occur at the same time; hence, there exists no race condition
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
