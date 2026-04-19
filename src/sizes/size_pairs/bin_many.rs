use crate::infallible::{Xap, XapBin};
use crate::sizes::{Bin, Many, size_pair::SizePair};
use core::iter::FusedIterator;

#[derive(Clone, Copy, Default)]
pub struct BinMany;

impl SizePair for BinMany {
    type S1 = Bin;

    type S2 = Many;

    type ThenBin = BinMany;

    type ThenMany = BinMany;

    // option

    type XapOptResult<M, X1, X2>
        = IterOptBinMany<<X2::Values as IntoIterator>::IntoIter>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        match x1.bin_value(i) {
            Some(Some(a)) => IterOptBinMany::success(Some(x2.xap(a).into_iter())),
            Some(None) => IterOptBinMany::fail(),
            None => IterOptBinMany::success(None),
        }
    }
}

// option - iter

pub enum IterOptBinMany<I: Iterator> {
    Success(Option<I>),
    Fail(bool),
}

impl<I: Iterator> IterOptBinMany<I> {
    pub fn success(i: Option<I>) -> Self {
        Self::Success(i)
    }

    pub fn fail() -> Self {
        Self::Fail(false)
    }
}

impl<I: Iterator> Iterator for IterOptBinMany<I> {
    type Item = Option<I::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Success(Some(iter)) => iter.next().map(Some),
            Self::Success(None) => None,
            Self::Fail(taken) => match taken {
                false => {
                    // SAFETY: error can be taken out only once; and on construction
                    // the error is not taken
                    *taken = true;
                    Some(None)
                }
                true => None, // the error is already taken and returned
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Success(Some(i)) => i.size_hint(),
            Self::Success(None) => (0, Some(0)),
            Self::Fail(false) => (1, Some(1)),
            Self::Fail(true) => (0, Some(0)),
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Success(Some(i)) => i.map(Some).fold(init, f),
            Self::Success(None) => init,
            Self::Fail(false) => f(init, None),
            Self::Fail(true) => init,
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self {
            Self::Success(Some(i)) => i.count(),
            Self::Success(None) => 0,
            Self::Fail(false) => 1,
            Self::Fail(true) => 0,
        }
    }
}

impl<I: FusedIterator> FusedIterator for IterOptBinMany<I> {}

impl<I: ExactSizeIterator> ExactSizeIterator for IterOptBinMany<I> {
    fn len(&self) -> usize {
        match self {
            Self::Success(Some(i)) => i.len(),
            Self::Success(None) => 0,
            Self::Fail(false) => 1,
            Self::Fail(true) => 0,
        }
    }
}
