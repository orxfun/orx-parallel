use crate::infallible::fun::flat_map::FlatMap;
use core::ops::Add;

pub struct FlatMapIterMany<I: Iterator, G: FlatMap<I = I::Item>> {
    i: I,
    g: G,
    inner: Option<<G::O as IntoIterator>::IntoIter>,
}

impl<I: Iterator, G: FlatMap<I = I::Item>> FlatMapIterMany<I, G>
where
    <G::O as IntoIterator>::Item: Add + core::iter::Sum<<G::O as IntoIterator>::Item>,
{
    fn sum_fold(self) -> <G::O as IntoIterator>::Item {
        let iter = self.i.map(|x| {
            self.g
                .flat_map(x)
                .into_iter()
                .sum::<<G::O as IntoIterator>::Item>()
        });
        iter.sum::<<G::O as IntoIterator>::Item>()
    }
}

impl<I: Iterator, G: FlatMap<I = I::Item>> FlatMapIterMany<I, G> {
    pub fn new(i: I, g: G) -> Self {
        let inner = None;
        Self { i, g, inner }
    }
}

impl<I: Iterator, G: FlatMap<I = I::Item>> Iterator for FlatMapIterMany<I, G> {
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
