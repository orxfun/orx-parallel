use crate::infallible::{Xap, XapBin};
use crate::option::size_pairs::SizePairOpt;
use crate::sizes::BinMany;

impl SizePairOpt for BinMany {
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

// iter

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
}
