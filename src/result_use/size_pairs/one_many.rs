use crate::infallible_use::{XapOne, XapUse};
use crate::result_use::size_pairs::size_pair_use_res::SizePairUseRes;
use crate::sizes::OneMany;
use core::iter::FusedIterator;

impl SizePairUseRes for OneMany {
    type XapUseResResult<M, E, X1, X2>
        = IterResOneMany<<X2::Values as IntoIterator>::IntoIter, E>
    where
        X1: XapUse<O = Result<M, E>, Size = Self::S1>,
        X2: XapUse<U = X1::U, I = M, Size = Self::S2>;

    #[inline]
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
        match x1.one_value(u, i) {
            Ok(a) => IterResOneMany::ok(x2.xap_use(u, a).into_iter()),
            Err(e) => IterResOneMany::err(e),
        }
    }
}

// iter

pub enum IterResOneMany<I: Iterator, E> {
    Success(I),
    Fail(Option<E>),
}

impl<I: Iterator, E> IterResOneMany<I, E> {
    pub fn ok(i: I) -> Self {
        Self::Success(i)
    }

    pub fn err(e: E) -> Self {
        Self::Fail(Some(e))
    }
}

impl<I: Iterator, E> Iterator for IterResOneMany<I, E> {
    type Item = Result<I::Item, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Success(iter) => iter.next().map(Ok),
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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Success(iter) => iter.size_hint(),
            Self::Fail(Some(_)) => (1, Some(1)),
            Self::Fail(None) => (0, Some(0)),
        }
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Success(iter) => iter.map(Ok).fold(init, f),
            Self::Fail(Some(e)) => f(init, Err(e)),
            Self::Fail(None) => init,
        }
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        match self {
            Self::Success(iter) => iter.count(),
            Self::Fail(Some(_)) => 1,
            Self::Fail(None) => 0,
        }
    }
}

impl<I: FusedIterator, E> FusedIterator for IterResOneMany<I, E> {}

impl<I: ExactSizeIterator, E> ExactSizeIterator for IterResOneMany<I, E> {
    fn len(&self) -> usize {
        match self {
            Self::Success(iter) => iter.len(),
            Self::Fail(Some(_)) => 1,
            Self::Fail(None) => 0,
        }
    }
}
