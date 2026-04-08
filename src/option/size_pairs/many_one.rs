use crate::infallible::{Xap, XapOne};
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::{ManyOne, One};

impl SizePairOpt for ManyOne {
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
        IterResManyOne { iter, x2: x2 }
    }
}

// iter

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
}
