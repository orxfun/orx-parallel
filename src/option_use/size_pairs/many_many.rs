use crate::infallible_use::XapUse;
use crate::option_use::size_pairs::size_pair_use_opt::SizePairUseOpt;
use crate::sizes::{Many, ManyMany};
use core::iter::FusedIterator;

impl SizePairUseOpt for ManyMany {
    type XapUseOptResult<M, X1, X2>
        = IterResManyMany<M, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    fn xap_use_res<M, X1, X2>(
        u: *mut X1::U,
        x1: X1,
        x2: X2,
        i: X1::I,
    ) -> Self::XapUseOptResult<M, X1, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>,
    {
        let iter = x1.xap_use(u, i).into_iter();
        let (x2, inner) = (x2, None);
        IterResManyMany { u, iter, x2, inner }
    }
}

// iter

pub struct IterResManyMany<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = Many>,
{
    u: *mut X2::U,
    iter: I,
    x2: X2,
    inner: Option<<X2::Values as IntoIterator>::IntoIter>,
}

impl<M, I, X2> Iterator for IterResManyMany<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = Many>,
{
    type Item = Option<X2::O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let elt @ Some(_) = and_then_or_clear(&mut self.inner, Iterator::next) {
                return elt.map(Some);
            }

            match self.iter.next() {
                Some(Some(i)) => self.inner = Some(self.x2.xap_use(self.u, i).into_iter()),
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
                Some(i) => {
                    agg = self
                        .x2
                        .xap_use(self.u, i)
                        .into_iter()
                        .map(Some)
                        .fold(agg, &mut f)
                }
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
                Some(i) => count += self.x2.xap_use(self.u, i).into_iter().count(),
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

impl<M, I, X2> FusedIterator for IterResManyMany<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = Many>,
    X2::Values: FusedIterator,
{
}
