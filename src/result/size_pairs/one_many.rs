use crate::infallible::{Xap, XapOne};
use crate::result::size_pairs::SizePairRes;
use crate::sizes::OneMany;
use core::iter::FusedIterator;

impl SizePairRes for OneMany {
    type XapResResult<M, E, X1, X2>
        = IterResOneMany<<X2::Values as IntoIterator>::IntoIter, E>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;

    #[inline(always)]
    fn xap_res<M, E, X1, X2>(x1: X1, x2: X2, i: X1::I) -> Self::XapResResult<M, E, X1, X2>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>,
    {
        match x1.one_value(i) {
            Ok(a) => IterResOneMany::ok(x2.xap(a).into_iter()),
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
}

impl<I: FusedIterator, E> FusedIterator for IterResOneMany<I, E> {}
