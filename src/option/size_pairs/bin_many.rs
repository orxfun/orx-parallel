use crate::infallible::{Xap, XapBin};
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::BinMany;

impl SizePairOpt for BinMany {
    type XapOptResult<M, X1, X2>
        = IterResBinMany<<X2::Values as IntoIterator>::IntoIter>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    fn xap_opt<M, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapOptResult<M, X1, X2>
    where
        X1: Xap<O = Option<M>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        match x1.bin_value(i) {
            Some(Some(a)) => IterResBinMany::success(Some(x2.xap(a).into_iter())),
            Some(None) => IterResBinMany::fail(),
            None => IterResBinMany::success(None),
        }
    }
}

// iter

pub enum IterResBinMany<I: Iterator> {
    Success(Option<I>),
    Fail(bool),
}

impl<I: Iterator> IterResBinMany<I> {
    pub fn success(i: Option<I>) -> Self {
        Self::Success(i)
    }

    pub fn fail() -> Self {
        Self::Fail(false)
    }
}

impl<I: Iterator> Iterator for IterResBinMany<I> {
    type Item = Option<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Success(Some(iter)) => iter.next().map(Some),
            Self::Success(None) => None,
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
