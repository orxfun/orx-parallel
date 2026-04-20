use crate::infallible::{Xap, XapOne};
use crate::infallible_use::{XapUse, XapUseOne};
use crate::sizes::size_pairs::{ManyBin, ManyMany};
use crate::sizes::{Many, One, size_pair::SizePair};
use core::iter::FusedIterator;

#[derive(Clone, Copy, Default)]
pub struct ManyOne;

impl SizePair for ManyOne {
    type S1 = Many;

    type S2 = One;

    type ThenBin = ManyBin;

    type ThenMany = ManyMany;

    // option

    type XapOptResult<M, X1, X2>
        = IterOptManyOne<M, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let iter = x1.xap(i).into_iter();
        IterOptManyOne { iter, x2: x2 }
    }

    // result

    type XapResResult<M, E, X1, X2>
        = IterResManyOne<M, E, <X1::Values as IntoIterator>::IntoIter, X2>
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
        IterResManyOne { iter, x2 }
    }

    // use - option

    type XapUseOptResult<M, X1, X2>
        = IterUseOptManyOne<M, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    #[inline]
    fn xap_use_opt<M, X1, X2>(
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
        IterUseOptManyOne { u, iter, x2 }
    }
}

// option - iter

pub struct IterOptManyOne<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: Xap<I = M, Size = One>,
{
    iter: I,
    x2: X2,
}

impl<M, I, X2> Iterator for IterOptManyOne<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: Xap<I = M, Size = One>,
{
    type Item = Option<X2::O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|a| a.map(|a| self.x2.one_value(a)))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut agg = init;

        for i in self.iter {
            match i {
                Some(i) => agg = f(agg, Some(self.x2.one_value(i))),
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

        for i in self.iter {
            match i {
                Some(_) => count += 1,
                None => return count,
            }
        }

        count
    }
}

impl<M, I, X2> FusedIterator for IterOptManyOne<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: Xap<I = M, Size = One>,
{
}

// result - iter

pub struct IterResManyOne<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = One>,
{
    iter: I,
    x2: X2,
}

impl<M, E, I, X2> Iterator for IterResManyOne<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = One>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|a| a.map(|a| self.x2.one_value(a)))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut agg = init;

        for i in self.iter {
            match i {
                Ok(i) => agg = f(agg, Ok(self.x2.one_value(i))),
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

        for i in self.iter {
            match i {
                Ok(_) => count += 1,
                Err(_) => return count,
            }
        }

        count
    }
}

impl<M, E, I, X2> FusedIterator for IterResManyOne<M, E, I, X2>
where
    I: FusedIterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = One>,
{
}

// use - option - iter

pub struct IterUseOptManyOne<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = One>,
{
    u: *mut X2::U,
    iter: I,
    x2: X2,
}

impl<M, I, X2> Iterator for IterUseOptManyOne<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = One>,
{
    type Item = Option<X2::O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|a| a.map(|a| self.x2.one_value(self.u, a)))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut agg = init;

        for i in self.iter {
            match i {
                Some(i) => agg = f(agg, Some(self.x2.one_value(self.u, i))),
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

        for i in self.iter {
            match i {
                Some(_) => count += 1,
                None => return count,
            }
        }

        count
    }
}

impl<M, I, X2> FusedIterator for IterUseOptManyOne<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = One>,
{
}
