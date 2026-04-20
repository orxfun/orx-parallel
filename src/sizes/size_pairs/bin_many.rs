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

    // result

    type XapResResult<M, E, X1, X2>
        = IterResBinMany<<X2::Values as IntoIterator>::IntoIter, E>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapResResult<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        match x1.bin_value(i) {
            Some(Ok(a)) => IterResBinMany::success(Some(x2.xap(a).into_iter())),
            Some(Err(e)) => IterResBinMany::fail(e),
            None => IterResBinMany::success(None),
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

// result - iter

pub enum IterResBinMany<I: Iterator, E> {
    Success(Option<I>),
    Fail(Option<E>),
}

impl<I: Iterator, E> IterResBinMany<I, E> {
    pub fn success(i: Option<I>) -> Self {
        Self::Success(i)
    }

    pub fn fail(e: E) -> Self {
        Self::Fail(Some(e))
    }
}

impl<I: Iterator, E> Iterator for IterResBinMany<I, E> {
    type Item = Result<I::Item, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Success(Some(iter)) => iter.next().map(Ok),
            Self::Success(None) => None,
            Self::Fail(e) => match e.is_some() {
                true => {
                    // SAFETY: error can be taken out only once; and on construction
                    // the error variant must be created with Some of an error
                    Some(Err(unsafe { e.take().unwrap_unchecked() }))
                }
                false => None, // the error is already taken and returned
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Success(Some(i)) => i.size_hint(),
            Self::Success(None) => (0, Some(0)),
            Self::Fail(_taken) => (1, Some(1)), // we will return only one element, the error
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Success(Some(i)) => i.map(Ok).fold(init, f),
            Self::Success(None) => init,
            Self::Fail(Some(e)) => f(init, Err(e)),
            Self::Fail(None) => init,
        }
    }

    fn count(self) -> usize {
        match self {
            Self::Success(Some(i)) => i.count(),
            Self::Success(None) => 0,
            Self::Fail(_taken) => 1, // we will return only one element, the error
        }
    }
}

impl<I: FusedIterator, E> FusedIterator for IterResBinMany<I, E> {}

impl<I: ExactSizeIterator, E> ExactSizeIterator for IterResBinMany<I, E> {
    fn len(&self) -> usize {
        match self {
            Self::Success(Some(i)) => i.len(),
            Self::Success(None) => 0,
            Self::Fail(_taken) => 1, // we will return only one element, the error
        }
    }
}
