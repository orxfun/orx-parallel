use crate::infallible::fun::Map;
use crate::infallible::sizes::{Bin, One};
use crate::infallible::{Xap, XapBin, XapOne};
use crate::option::xap_opt::XapOpt;
use crate::option::xap_opt_variants::{XapOptBinBin, XapOptBinMany};

pub struct XapOptBinOne<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
    x1: X1,
    x2: X2,
}

impl<M, X1, X2> Clone for XapOptBinOne<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, X1, X2> Copy for XapOptBinOne<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
}

unsafe impl<M, X1, X2> Send for XapOptBinOne<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
}

impl<M, X1, X2> XapOptBinOne<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, X1, X2> XapOpt for XapOptBinOne<M, X1, X2>
where
    X1: Xap<O = Option<M>, Size = Bin>,
    X2: Xap<I = M, Size = One>,
{
    type I = X1::I;

    type M = M;

    type O = X2::O;

    type Results = Option<Option<X2::O>>;

    #[inline(always)]
    fn xap_res(&self, i: Self::I) -> Self::Results {
        let a = self.x1.bin_value(i);
        a.map(|a| a.map(|a| self.x2.one_value(a)))
    }

    // transformations

    type Map<Q, H>
        = XapOptBinOne<M, X1, X2::Map<Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        XapOptBinOne::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapOptBinOne<M, X1, X2::Inspect<H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        XapOptBinOne::new(self.x1, self.x2.inspect(h))
    }

    type Filter<H>
        = XapOptBinBin<M, X1, X2::Filter<H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        XapOptBinBin::new(self.x1, self.x2.filter(h))
    }

    type FilterMap<Q, H>
        = XapOptBinBin<M, X1, X2::FilterMap<Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        XapOptBinBin::new(self.x1, self.x2.filter_map(h))
    }

    type FlatMap<V, H>
        = XapOptBinMany<M, X1, X2::FlatMap<V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        XapOptBinMany::new(self.x1, self.x2.flat_map(h))
    }

    // transformations - helper

    type Mapped<H>
        = XapOptBinOne<M, X1, X2::Mapped<H>>
    where
        H: Map<I = Self::O>;

    fn mapped<H>(self, h: H) -> Self::Mapped<H>
    where
        H: Map<I = Self::O>,
    {
        XapOptBinOne::new(self.x1, self.x2.mapped(h))
    }
}
