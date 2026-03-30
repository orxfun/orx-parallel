use crate::infallible::size::Bin;
use crate::infallible::xap::{Xap, XapBin};
use crate::result::xap_res::{ResOf, XapRes};

pub struct XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Bin>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Bin>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Bin>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = Option<ResOf<Self>>;

    #[inline(always)]
    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Self::Results {
        self.x1.bin_value(i).and_then(|a| match a {
            Ok(a) => self.x2.bin_value(a).map(Ok),
            Err(e) => Some(Err(e)),
        })
    }

    // transformations

    type Map<Q, H>
        = XapResBinBin<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send,
    {
        XapResBinBin::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapResBinBin<M, E, X1, X2::Inspect<H>>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send,
    {
        XapResBinBin::new(self.x1, self.x2.inspect(h))
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
}
