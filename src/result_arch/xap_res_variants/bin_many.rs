use crate::infallible::{Many, Xap, ZeroOne};
use crate::result::xap_res::{ResOf, XapRes};
use core::iter::Flatten;
use core::option::IntoIter;

pub struct XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = Many>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = Many>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = Many>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = IterResBinMany<<<X2 as Xap>::Values as IntoIterator>::IntoIter, E>;

    fn xap_res(&self, i: Self::I) -> Self::Results {
        match self.x1.xap(i).into_iter().next() {
            Some(Ok(a)) => IterResBinMany::ok(Some(self.x2.xap(a).into_iter())),
            Some(Err(e)) => IterResBinMany::err(e),
            None => IterResBinMany::ok(None),
        }
    }
}

// iter

pub enum IterResBinMany<I: Iterator, E> {
    Ok(Option<I>),
    Err(Option<E>),
}

impl<I: Iterator, E> IterResBinMany<I, E> {
    pub fn ok(i: Option<I>) -> Self {
        Self::Ok(i)
    }

    pub fn err(e: E) -> Self {
        Self::Err(Some(e))
    }
}

impl<I: Iterator, E> Iterator for IterResBinMany<I, E> {
    type Item = Result<I::Item, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Ok(Some(iter)) => iter.next().map(Ok),
            Self::Ok(None) => None,
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
