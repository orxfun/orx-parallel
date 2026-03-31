use crate::infallible::{Many, One, Xap};
use crate::result::xap_res::XapRes;

pub struct XapResOneMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = One>,
    X2: Xap<I = M, Count = Many>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResOneMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = One>,
    X2: Xap<I = M, Count = Many>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResOneMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = One>,
    X2: Xap<I = M, Count = Many>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = IterResOneMany<<<X2 as Xap>::Values as IntoIterator>::IntoIter, E>;

    #[inline]
    fn xap_res(&self, i: Self::I) -> Self::Results {
        match unsafe { self.x1.xap(i).into_iter().next().unwrap_unchecked() } {
            Ok(a) => IterResOneMany::ok(self.x2.xap(a).into_iter()),
            Err(e) => IterResOneMany::err(e),
        }
    }
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
