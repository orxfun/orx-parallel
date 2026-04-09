use crate::infallible_use::{XapBin, XapUse};
use crate::result_use::size_pairs::size_pair_use_res::SizePairUseRes;
use crate::sizes::{Bin, ManyBin};
use core::iter::FusedIterator;

impl SizePairUseRes for ManyBin {
    type XapUseResResult<M, E, X1, X2>
        = IterResManyBin<M, E, <X1::Values as IntoIterator>::IntoIter, X2>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

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
        IterResManyBin { u, iter, x2 }
    }
}

// iter

pub struct IterResManyBin<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: XapUse<I = M, Size = Bin>,
{
    u: *mut <X2 as XapUse>::U,
    iter: I,
    x2: X2,
}

impl<M, E, I, X2> Iterator for IterResManyBin<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: XapUse<I = M, Size = Bin>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Ok(a)) => {
                    let b = self.x2.bin_value(self.u, a);
                    if b.is_some() {
                        return b.map(Ok);
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
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
                Ok(i) => {
                    let i = x2.bin_value(self.u, i);
                    if let Some(i) = i {
                        agg = f(agg, Ok(i))
                    }
                }
                Err(e) => return f(agg, Err(e)),
            }
        }
        agg
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.take_while(|x| x.is_ok()).count()
    }
}

impl<M, E, I, X2> FusedIterator for IterResManyBin<M, E, I, X2>
where
    I: FusedIterator<Item = Result<M, E>>,
    X2: XapUse<I = M, Size = Bin>,
{
}
