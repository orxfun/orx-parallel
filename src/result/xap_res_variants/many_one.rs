use crate::infallible::fun::Map;
use crate::infallible::sizes::{Many, One};
use crate::infallible::{Xap, XapOne};
use crate::result::xap_res::XapRes;
use crate::result::xap_res_variants::{XapResManyBin, XapResManyMany};

pub struct XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = One>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> Clone for XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = One>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> Copy for XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = One>,
{
}

unsafe impl<M, E, X1, X2> Send for XapResManyOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = One>,
{
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
    type I = X1::I;

    type M = M;

    type E = E;

    type O = X2::O;

    type Results = IterResManyOne<M, E, <<X1 as Xap>::Values as IntoIterator>::IntoIter, X2>;

    fn xap_res(&self, i: Self::I) -> Self::Results {
        let iter = self.x1.xap(i).into_iter();
        IterResManyOne { iter, x2: self.x2 }
    }

    // transformations

    type Map<Q, H>
        = XapResManyOne<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        XapResManyOne::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapResManyOne<M, E, X1, X2::Inspect<H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        XapResManyOne::new(self.x1, self.x2.inspect(h))
    }

    type Filter<H>
        = XapResManyBin<M, E, X1, X2::Filter<H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        XapResManyBin::new(self.x1, self.x2.filter(h))
    }

    type FilterMap<Q, H>
        = XapResManyBin<M, E, X1, X2::FilterMap<Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        XapResManyBin::new(self.x1, self.x2.filter_map(h))
    }

    type FlatMap<V, H>
        = XapResManyMany<M, E, X1, X2::FlatMap<V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        XapResManyMany::new(self.x1, self.x2.flat_map(h))
    }

    // transformations - helper

    type Mapped<H>
        = XapResManyOne<M, E, X1, X2::Mapped<H>>
    where
        H: Map<I = Self::O>;

    fn mapped<H>(self, h: H) -> Self::Mapped<H>
    where
        H: Map<I = Self::O>,
    {
        XapResManyOne::new(self.x1, self.x2.mapped(h))
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
