use crate::infallible::Xap;
use crate::infallible::sizes::{Many, One};
use crate::result::size_pairs::SizePair;

pub struct OneMany;

impl SizePair for OneMany {
    type S1 = One;

    type S2 = Many;

    type Results<M, E, X1, X2>
        = IterResOneMany<<X2::Values as IntoIterator>::IntoIter, E>
    where
        X1: Xap<O = Result<M, E>, Size = Self::S1>,
        X2: Xap<I = M, Size = Self::S2>;
}

// iter

pub enum IterResOneMany<I: Iterator, E> {
    Ok(I),
    Err(Option<E>),
}

impl<I: Iterator, E> IterResOneMany<I, E> {
    pub fn ok(i: I) -> Self {
        Self::Ok(i)
    }

    pub fn err(e: E) -> Self {
        Self::Err(Some(e))
    }
}

impl<I: Iterator, E> Iterator for IterResOneMany<I, E> {
    type Item = Result<I::Item, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Ok(iter) => iter.next().map(Ok),
            Self::Err(e) => match e.is_some() {
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
