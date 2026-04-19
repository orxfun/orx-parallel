use crate::infallible::{Xap, XapBin};
use crate::sizes::{Bin, Many, size_pair::SizePair, size_pairs::ManyMany};
use core::iter::FusedIterator;

#[derive(Clone, Copy, Default)]
pub struct ManyBin;

impl SizePair for ManyBin {
    type S1 = Many;

    type S2 = Bin;

    type ThenBin = ManyBin;

    type ThenMany = ManyMany;

    // option

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

// option - iter

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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // all bin choices may fail (lb=0)
        // but we can't have more than the source (ub=iter.ub)
        let (_, ub) = self.iter.size_hint();
        (0, ub)
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let (iter, x2) = (self.iter, self.x2);
        let mut agg = init;
        for i in iter {
            match i {
                Some(i) => {
                    let i = x2.bin_value(i);
                    if i.is_some() {
                        agg = f(agg, i)
                    }
                }
                None => return f(agg, None),
            }
        }
        agg
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.take_while(|x| x.is_some()).count()
    }
}

impl<M, I, X2> FusedIterator for IterOptManyBin<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: Xap<I = M, Size = Bin>,
{
}
