use crate::infallible_use::{XapUse, XapUseBin};
use crate::option_use::size_pairs::size_pair_use_opt::SizePairUseOpt;
use crate::sizes::{Bin, ManyBin};
use core::iter::FusedIterator;

impl SizePairUseOpt for ManyBin {
    type XapUseOptResult<M, X1, X2>
        = IterResManyBin<M, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

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
        IterResManyBin { u, iter, x2 }
    }
}

// iter

pub struct IterResManyBin<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = Bin>,
{
    u: *mut <X2 as XapUse>::U,
    iter: I,
    x2: X2,
}

impl<M, I, X2> Iterator for IterResManyBin<M, I, X2>
where
    I: Iterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = Bin>,
{
    type Item = Option<X2::O>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Some(a)) => {
                    let b = self.x2.bin_value(self.u, a);
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
                    let i = x2.bin_value(self.u, i);
                    if let Some(i) = i {
                        agg = f(agg, Some(i))
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

impl<M, I, X2> FusedIterator for IterResManyBin<M, I, X2>
where
    I: FusedIterator<Item = Option<M>>,
    X2: XapUse<I = M, Size = Bin>,
{
}
