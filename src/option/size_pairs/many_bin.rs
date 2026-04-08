use crate::infallible::{Xap, XapBin};
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::{Bin, ManyBin};
use core::iter::FusedIterator;

impl SizePairOpt for ManyBin {
    type XapOptResult<M, X1, X2>
        = IterOptManyBin<M, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let iter = x1.xap(i).into_iter();
        IterOptManyBin { iter, x2 }
    }
}

// iter

pub struct IterOptManyBin<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Bin>,
{
    iter: I,
    x2: X2,
}

impl<M, I, X2> Iterator for IterOptManyBin<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Bin>,
{
    type Item = Option<X2::O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Some(a)) => {
                    let b = self.x2.bin_value(a);
                    if b.is_some() {
                        return b.map(Some);
                    }
                }
                Some(None) => return Some(None),
                None => return None,
            }
        }
    }
}

impl<M, I, X2> FusedIterator for IterOptManyBin<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Bin>,
{
}
