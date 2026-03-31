use crate::infallible::fun::Map;
use crate::infallible::size::{Bin, One};
use crate::infallible::{Xap, XapBin, XapOne};
use crate::result::xap_res::{ResOf, XapRes};
use crate::result::xap_res_variants::{XapResBinBin, XapResBinMany};

pub struct XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> Clone for XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> Copy for XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
}

unsafe impl<M, E, X1, X2> Send for XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
}

impl<M, E, X1, X2> XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
    type I = X1::I;

    type M = M;

    type E = E;

    type O = X2::O;

    type Results = Option<ResOf<Self>>;

    #[inline(always)]
    fn xap_res(&self, i: Self::I) -> Self::Results {
        let a = self.x1.bin_value(i);
        a.map(|a| a.map(|a| self.x2.one_value(a)))
    }

    // transformations

    type Map<Q, H>
        = XapResBinOne<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        XapResBinOne::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapResBinOne<M, E, X1, X2::Inspect<H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        XapResBinOne::new(self.x1, self.x2.inspect(h))
    }

    type Filter<H>
        = XapResBinBin<M, E, X1, X2::Filter<H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        XapResBinBin::new(self.x1, self.x2.filter(h))
    }

    type FilterMap<Q, H>
        = XapResBinBin<M, E, X1, X2::FilterMap<Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        XapResBinBin::new(self.x1, self.x2.filter_map(h))
    }

    type FlatMap<V, H>
        = XapResBinMany<M, E, X1, X2::FlatMap<V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        XapResBinMany::new(self.x1, self.x2.flat_map(h))
    }

    // transformations - helper

    type Mapped<H>
        = XapResBinOne<M, E, X1, X2::Mapped<H>>
    where
        H: Map<I = Self::O>;

    fn mapped<H>(self, h: H) -> Self::Mapped<H>
    where
        H: Map<I = Self::O>,
    {
        XapResBinOne::new(self.x1, self.x2.mapped(h))
    }
}
