use crate::infallible::fun::Map;
use crate::infallible::sizes::{Bin, Many};
use crate::infallible::{MapOf, Xap, XapBin};
use crate::result::xap_res::XapRes;
use crate::result::xap_res_variants::XapResManyMany;

pub struct XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> Clone for XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> Copy for XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
}

unsafe impl<M, E, X1, X2> Send for XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
}

impl<M, E, X1, X2> XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResManyBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Many>,
    X2: Xap<I = M, Size = Bin>,
{
    type I = X1::I;

    type M = M;

    type E = E;

    type O = X2::O;

    type Results = IterResManyBin<M, E, <<X1 as Xap>::Values as IntoIterator>::IntoIter, X2>;

    fn xap_res(&self, i: Self::I) -> Self::Results {
        let iter = self.x1.xap(i).into_iter();
        IterResManyBin { iter, x2: self.x2 }
    }

    // transformations

    type Map<Q, H>
        = XapResManyBin<M, E, X1, MapOf<X2, Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        XapResManyBin::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapResManyBin<M, E, X1, X2::Inspect<H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        XapResManyBin::new(self.x1, self.x2.inspect(h))
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
        = XapResManyBin<M, E, X1, X2::Mapped<H>>
    where
        H: Map<I = Self::O>;

    fn mapped<H>(self, h: H) -> Self::Mapped<H>
    where
        H: Map<I = Self::O>,
    {
        XapResManyBin::new(self.x1, self.x2.mapped(h))
    }
}

// iter

pub struct IterResManyBin<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Bin>,
{
    iter: I,
    x2: X2,
}

impl<M, E, I, X2> Iterator for IterResManyBin<M, E, I, X2>
where
    I: Iterator<Item = Result<M, E>>,
    X2: Xap<I = M, Size = Bin>,
{
    type Item = Result<X2::O, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Ok(a)) => {
                    let b = self.x2.bin_value(a);
                    if b.is_some() {
                        return b.map(Ok);
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }
}
