use crate::infallible_use::{XapUse, XapUseOne};
use crate::option_use::size_pairs::size_pair_use_opt::SizePairUseOpt;
use crate::sizes::OneMany;
use core::iter::FusedIterator;

impl SizePairUseOpt for OneMany {
    type XapUseOptResult<M, X1, X2>
        = IterResOneMany<<X2::Values as IntoIterator>::IntoIter>
    where
        X1: XapUse<O = Option<M>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    #[inline]
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
        match x1.one_value(u, i) {
            Some(a) => IterResOneMany::ok(x2.xap_use(u, a).into_iter()),
            None => IterResOneMany::err(),
        }
    }
}

// iter

pub enum IterResOneMany<I: Iterator> {
    Success(I),
    Fail(bool),
}

impl<I: Iterator> IterResOneMany<I> {
    pub fn ok(i: I) -> Self {
        Self::Success(i)
    }

    pub fn err() -> Self {
        Self::Fail(false)
    }
}

impl<I: Iterator> Iterator for IterResOneMany<I> {
    type Item = Option<I::Item>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Success(iter) => iter.next().map(Some),
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

impl<I: FusedIterator> FusedIterator for IterResOneMany<I> {}

impl<I: ExactSizeIterator> ExactSizeIterator for IterResOneMany<I> {
    fn len(&self) -> usize {
        match self {
            Self::Success(iter) => iter.len(),
            Self::Fail(false) => 1,
            Self::Fail(true) => 0,
        }
    }
}
