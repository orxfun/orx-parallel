use crate::infallible::size::{Many, One};
use crate::infallible::xap::{Xap, XapOne};
use crate::result::xap_res::XapRes;

pub struct XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = One>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = One>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = One>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = IterResManyOne<M, E, <<X1 as Xap>::Values as IntoIterator>::IntoIter, X2>;

    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Self::Results {
        let iter = self.x1.xap(i).into_iter();
        IterResManyOne { iter, x2: self.x2 }
    }

    // transformations

    type Map<Q, H>
        = XapResManyOne<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send,
    {
        XapResManyOne::new(self.x1, self.x2.map(h))
    }
}

// iter

pub struct IterResManyOne<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = One>,
{
    iter: I,
    x2: X2,
}

impl<M, E, I, X2> Iterator for IterResManyOne<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = One>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|a| a.map(|a| self.x2.one_value(a)))
    }
}
