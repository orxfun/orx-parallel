use crate::infallible::sizes::{Bin, Many};
use crate::infallible::{Xap, XapBin};
use crate::result::size_pairs::SizePair;

pub struct ManyBin;

impl SizePair for ManyBin {
    type S1 = Many;

    type S2 = Bin;

    type Results<M, E, X1, X2>
        = IterResManyBin<M, E, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::Results<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        let iter = x1.xap(i).into_iter();
        IterResManyBin { iter, x2 }
    }
}

// iter

pub struct IterResManyBin<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Bin>,
{
    iter: I,
    x2: X2,
}

impl<M, E, I, X2> Iterator for IterResManyBin<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Bin>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Ok(a)) => {
                    let b = self.x2.bin_value(a);
                    if b.is_some() {
                        return b.map(Ok);
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }
}
