use crate::infallible_use::{XapUse, XapUseOne};
use crate::option_use::size_pairs::size_pair_use_opt::SizePairUseRes;
use crate::sizes::{ManyOne, One};
use core::iter::FusedIterator;

impl SizePairUseRes for ManyOne {
    type XapUseResResult<M, X1, X2>
        = IterResManyOne<M, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    #[inline]
    fn xap_use_res<M, X1, X2>(
        u: *mut X1::U,
        x1: X1,
        x2: X2,
        i: X1::I,
    ) -> Self::XapUseResResult<M, X1, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>,
    {
        let iter = x1.xap_use(u, i).into_iter();
        IterResManyOne { u, iter, x2 }
    }
}

// iter

pub struct IterResManyOne<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = One>,
{
    u: *mut X2::U,
    iter: I,
    x2: X2,
}

impl<M, I, X2> Iterator for IterResManyOne<M, I, X2>
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

impl<M, I, X2> FusedIterator for IterResManyOne<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = One>,
{
}
