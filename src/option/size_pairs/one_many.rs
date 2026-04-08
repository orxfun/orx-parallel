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
}

impl<I: FusedIterator> FusedIterator for IterOptOneMany<I> {}
