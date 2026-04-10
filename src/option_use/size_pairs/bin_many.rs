use crate::infallible_use::{XapUse, XapUseBin};
use crate::option_use::size_pairs::SizePairUseOpt;
use crate::sizes::BinMany;
use core::iter::FusedIterator;

impl SizePairUseOpt for BinMany {
    type XapUseOptResult<M, X1, X2>
        = IterResBinMany<<X2::Values as IntoIterator>::IntoIter>
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
        match x1.bin_value(u, i) {
            Some(Some(a)) => IterResBinMany::success(Some(x2.xap_use(u, a).into_iter())),
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

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Success(Some(iter)) => iter.next().map(Some),
            Self::Success(None) => None,
            Self::Fail(taken) => match taken {
                false => {
                    // SAFETY: error can be taken out only once; and on construction
                    // the error variant must be created with Some of an error
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

    fn count(self) -> usize {
        match self {
            Self::Success(Some(i)) => i.count(),
            Self::Success(None) => 0,
            Self::Fail(false) => 1,
            Self::Fail(true) => 0,
        }
    }
}

impl<I: FusedIterator> FusedIterator for IterResBinMany<I> {}

impl<I: ExactSizeIterator> ExactSizeIterator for IterResBinMany<I> {
    fn len(&self) -> usize {
        match self {
            Self::Success(Some(i)) => i.len(),
            Self::Success(None) => 0,
            Self::Fail(false) => 1,
            Self::Fail(true) => 0,
        }
    }
}
