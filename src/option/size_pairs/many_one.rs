use crate::infallible::{Xap, XapOne};
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::{ManyOne, One};
use core::iter::FusedIterator;

impl SizePairOpt for ManyOne {
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
}

// iter

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
}

impl<M, I, X2> FusedIterator for IterOptManyOne<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: Xap<I = M, Size = One>,
{
}
