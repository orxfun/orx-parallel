use crate::infallible_use::XapUse;
use crate::option_use::size_pairs::size_pair_use_opt::SizePairUseRes;
use crate::sizes::{Many, ManyMany};
use core::iter::FusedIterator;

impl SizePairUseRes for ManyMany {
    type XapUseResResult<M, E, X1, X2>
        = IterResManyMany<M, E, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    fn xap_use_res<M, E, X1, X2>(
        u: *mut X1::U,
        x1: X1,
        x2: X2,
        i: X1::I,
    ) -> Self::XapUseResResult<M, E, X1, X2>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>,
    {
        let iter = x1.xap_use(u, i).into_iter();
        let (x2, inner) = (x2, None);
        IterResManyMany { u, iter, x2, inner }
    }
}

// iter

pub struct IterResManyMany<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: XapUse<I = M, Size = Many>,
{
    u: *mut X2::U,
    iter: I,
    x2: X2,
    inner: Option<<X2::Values as IntoIterator>::IntoIter>,
}

impl<M, E, I, X2> Iterator for IterResManyMany<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: XapUse<I = M, Size = Many>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt.map(Ok);
            }

            match self.iter.next() {
                Some(Ok(i)) => self.inner = Some(self.x2.xap_use(self.u, i).into_iter()),
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
                Ok(i) => {
                    agg = self
                        .x2
                        .xap_use(self.u, i)
                        .into_iter()
                        .map(Ok)
                        .fold(agg, &mut f)
                }
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
                Ok(i) => count += self.x2.xap_use(self.u, i).into_iter().count(),
                Err(_) => return count,
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

impl<M, E, I, X2> FusedIterator for IterResManyMany<M, E, I, X2>
where
    I: FusedIterator<Item = Result<M, E>>,
    X2: XapUse<I = M, Size = Many>,
    X2::Values: FusedIterator,
{
}
