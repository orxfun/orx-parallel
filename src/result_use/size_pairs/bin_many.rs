use crate::infallible_use::{XapUse, XapUseBin};
use crate::result_use::size_pairs::size_pair_use_res::SizePairUseRes;
use crate::sizes::BinMany;
use core::iter::FusedIterator;

impl SizePairUseRes for BinMany {
    type XapUseResResult<M, E, X1, X2>
        = IterResBinMany<<X2::Values as IntoIterator>::IntoIter, E>
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
        match x1.bin_value(u, i) {
            Some(Ok(a)) => IterResBinMany::success(Some(x2.xap_use(u, a).into_iter())),
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
