use crate::infallible::Xap;
use crate::sizes::{Many, size_pair::SizePair};
use core::iter::FusedIterator;

#[derive(Clone, Copy, Default)]
pub struct ManyMany;

impl SizePair for ManyMany {
    type S1 = Many;

    type S2 = Many;

    type ThenBin = ManyMany;

    type ThenMany = ManyMany;

    // option

    type XapOptResult<M, X1, X2>
        = IterOptManyMany<M, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let iter = x1.xap(i).into_iter();
        let (x2, inner) = (x2, None);
        IterOptManyMany { iter, x2, inner }
    }

    // result

    type XapResResult<M, E, X1, X2>
        = IterResManyMany<M, E, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapResResult<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let iter = x1.xap(i).into_iter();
        let (x2, inner) = (x2, None);
        IterResManyMany { iter, x2, inner }
    }
}

// option - iter

pub struct IterOptManyMany<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Many>,
{
    iter: I,
    x2: X2,
    inner: Option<<X2::Values as IntoIterator>::IntoIter>,
}

impl<M, I, X2> Iterator for IterOptManyMany<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Many>,
{
    type Item = Option<X2::O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt.map(Some);
            }

            match self.iter.next() {
                Some(Some(i)) => self.inner = Some(self.x2.xap(i).into_iter()),
                Some(None) => return Some(None),
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

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut agg = init;

        if let Some(iter) = self.inner {
            agg = iter.map(Some).fold(agg, &mut f);
        }

        for i in self.iter {
            match i {
                Some(i) => agg = self.x2.xap(i).into_iter().map(Some).fold(agg, &mut f),
                None => return f(agg, None),
            }
        }

        agg
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let mut count = 0;

        if let Some(iter) = self.inner {
            count += iter.count();
        }

        for i in self.iter {
            match i {
                Some(i) => count += self.x2.xap(i).into_iter().count(),
                None => return count,
            }
        }

        count
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

impl<M, I, X2> FusedIterator for IterOptManyMany<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Many>,
    X2::Values: FusedIterator,
{
}

// result - iter

pub struct IterResManyMany<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Many>,
{
    iter: I,
    x2: X2,
    inner: Option<<X2::Values as IntoIterator>::IntoIter>,
}

impl<M, E, I, X2> Iterator for IterResManyMany<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Many>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt.map(Ok);
            }

            match self.iter.next() {
                Some(Ok(i)) => self.inner = Some(self.x2.xap(i).into_iter()),
                Some(Err(e)) => return Some(Err(e)),
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

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut agg = init;

        if let Some(iter) = self.inner {
            agg = iter.map(Ok).fold(agg, &mut f);
        }

        for i in self.iter {
            match i {
                Ok(i) => agg = self.x2.xap(i).into_iter().map(Ok).fold(agg, &mut f),
                Err(e) => return f(agg, Err(e)),
            }
        }

        agg
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        let mut count = 0;

        if let Some(iter) = self.inner {
            count += iter.count();
        }

        for i in self.iter {
            match i {
                Ok(i) => count += self.x2.xap(i).into_iter().count(),
                Err(_) => return count,
            }
        }

        count
    }
}

impl<M, E, I, X2> FusedIterator for IterResManyMany<M, E, I, X2>
where
    I: FusedIterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Many>,
    X2::Values: FusedIterator,
{
}
