use crate::xap::Xap;

pub struct XapIter<I: Iterator, X: Xap<I = I::Item>> {
    i: I,
    x: X,
    inner: Option<<X::Values as IntoIterator>::IntoIter>,
}

impl<I: Iterator, X: Xap<I = I::Item>> XapIter<I, X> {
    pub fn new(i: I, x: X) -> Self {
        let inner = None;
        Self { i, x, inner }
    }
}

impl<I: Iterator, X: Xap<I = I::Item>> Iterator for XapIter<I, X> {
    type Item = X::O;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt;
            }

            match self.i.next() {
                Some(i) => self.inner = Some(self.x.xap(i).into_iter()),
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

        self.i
            .fold(acc, |acc, i| self.x.xap(i).into_iter().fold(acc, &mut f))
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

        self.i
            .fold(count, |count, i| count + self.x.xap(i).into_iter().count())
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
