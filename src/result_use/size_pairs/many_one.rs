use crate::infallible_use::{XapOne, XapUse};
use crate::result_use::size_pairs::size_pair_use_res::SizePairUseRes;
use crate::sizes::{ManyOne, One};
use core::iter::FusedIterator;

impl SizePairUseRes for ManyOne {
    type XapUseResResult<M, E, X1, X2>
        = IterResManyOne<M, E, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    #[inline]
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
        IterResManyOne { u, iter, x2 }
    }
}

// iter

pub struct IterResManyOne<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: XapUse<I = M, Size = One>,
{
    u: *mut X2::U,
    iter: I,
    x2: X2,
}

impl<M, E, I, X2> Iterator for IterResManyOne<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: XapUse<I = M, Size = One>,
{
    type Item = Result<X2::O, E>;

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
                Ok(i) => agg = f(agg, Ok(self.x2.one_value(self.u, i))),
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
    X2: XapUse<I = M, Size = One>,
{
}
