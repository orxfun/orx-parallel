use crate::infallible::fun::Map;
use crate::infallible::size::{Bin, One};
use crate::infallible::xap::{Xap, XapBin, XapOne};
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
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = Option<ResOf<Self>>;

    #[inline(always)]
    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Self::Results {
        let a = self.x1.bin_value(i);
        a.map(|a| a.map(|a| self.x2.one_value(a)))
    }

    // transformations

    type Map<Q, H>
        = XapResBinOne<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send,
    {
        XapResBinOne::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapResBinOne<M, E, X1, X2::Inspect<H>>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send,
    {
        XapResBinOne::new(self.x1, self.x2.inspect(h))
    }

    type Filter<H>
        = XapResBinBin<M, E, X1, X2::Filter<H>>
    where
        H: Fn(&<Self::X2 as Xap>::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&<Self::X2 as Xap>::O) -> bool + Copy + Send,
    {
        XapResBinBin::new(self.x1, self.x2.filter(h))
    }

    type FilterMap<Q, H>
        = XapResBinBin<M, E, X1, X2::FilterMap<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Option<Q> + Copy + Send,
    {
        XapResBinBin::new(self.x1, self.x2.filter_map(h))
    }

    type FlatMap<V, H>
        = XapResBinMany<M, E, X1, X2::FlatMap<V, H>>
    where
        V: IntoIterator,
        H: Fn(<Self::X2 as Xap>::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(<Self::X2 as Xap>::O) -> V + Copy + Send,
    {
        XapResBinMany::new(self.x1, self.x2.flat_map(h))
    }

    // transformations - helper

    type Mapped<H>
        = XapResBinOne<M, E, X1, X2::Mapped<H>>
    where
        H: Map<I = <Self::X2 as Xap>::O>;

    fn mapped<H>(self, h: H) -> Self::Mapped<H>
    where
        H: Map<I = <Self::X2 as Xap>::O>,
    {
        XapResBinOne::new(self.x1, self.x2.mapped(h))
    }
}
