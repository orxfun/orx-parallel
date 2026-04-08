use crate::infallible::{Xap, XapOne};
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::OneMany;
use core::iter::FusedIterator;

impl SizePairOpt for OneMany {
    type XapOptResult<M, X1, X2>
        = IterOptOneMany<<X2::Values as IntoIterator>::IntoIter>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        match x1.one_value(i) {
            Some(a) => IterOptOneMany::ok(x2.xap(a).into_iter()),
            None => IterOptOneMany::err(),
        }
    }
}

// iter

pub enum IterOptOneMany<I: Iterator> {
    Success(I),
    Fail(bool),
}

impl<I: Iterator> IterOptOneMany<I> {
    pub fn ok(i: I) -> Self {
        Self::Success(i)
    }

    pub fn err() -> Self {
        Self::Fail(false)
    }
}

impl<I: Iterator> Iterator for IterOptOneMany<I> {
    type Item = Option<I::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Success(iter) => iter.next().map(Some),
            Self::Fail(taken) => match taken {
                true => {
                    // SAFETY: error can be taken out only once; and on construction
                    // the error is not taken
                    *taken = true;
                    Some(None)
                }
                false => None, // the error is already taken and returned
            },
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Success(iter) => iter.size_hint(),
            Self::Fail(false) => (1, Some(1)),
            Self::Fail(true) => (0, Some(0)),
        }
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Success(iter) => iter.map(Some).fold(init, f),
            Self::Fail(false) => f(init, None),
            Self::Fail(true) => init,
        }
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self {
            Self::Success(iter) => iter.count(),
            Self::Fail(false) => 1,
            Self::Fail(true) => 0,
        }
    }
}

impl<I: FusedIterator> FusedIterator for IterOptOneMany<I> {}

impl<I: ExactSizeIterator> ExactSizeIterator for IterOptOneMany<I> {
    fn len(&self) -> usize {
        match self {
            Self::Success(iter) => iter.len(),
            Self::Fail(false) => 1,
            Self::Fail(true) => 0,
        }
    }
}
