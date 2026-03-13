use crate::xap::fun::flat_map::FlatMap;
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
        todo!();
        let (flo, fhi) = self
            .inner
            .as_ref()
            .map_or((0, Some(0)), <G::O as IntoIterator>::IntoIter::size_hint);

        // (flo, None)

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
        let mut acc = match self.inner {
            Some(inner) => inner.count(),
            None => 0,
        };

        for i in self.i {
            let inner = self.g.flat_map(i);
            acc += inner.into_iter().count();
        }

        acc
    }

    // fn flatten(self) -> core::iter::Flatten<Self>
    // where
    //     Self: Sized,
    //     Self::Item: IntoIterator,
    // {
    //     let mut abc = [1, 2, 3].into_iter().map(|x| [x, x + 1]);
    //     let def = abc.flatten();
    //     todo!()
    // }
}

#[inline(always)]
fn and_then_or_clear<T, U>(opt: &mut Option<T>, f: impl FnOnce(&mut T) -> Option<U>) -> Option<U> {
    let x = f(opt.as_mut()?);
    if x.is_none() {
        *opt = None;
    }
    x
}
