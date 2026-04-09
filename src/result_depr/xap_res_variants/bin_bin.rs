use crate::infallible::fun::Map;
use crate::infallible::sizes::Bin;
use crate::infallible::{MapOf, Xap, XapBin};
use crate::result_depr::xap_res::{InOf, ResOf, XapRes};
use crate::result_depr::xap_res_variants::XapResBinMany;

pub struct XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Bin>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> Clone for XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Bin>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> Copy for XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Bin>,
{
}

unsafe impl<M, E, X1, X2> Send for XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = Bin>,
    X2: Xap<I = M, Size = Bin>,
{
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

    type Size = Bin;

    type Results = Option<ResOf<Self>>;

    #[inline(always)]
    fn xap_res(&self, i: InOf<Self>) -> Self::Results {
        self.x1.bin_value(i).and_then(|a| match a {
            Ok(a) => self.x2.bin_value(a).map(Ok),
            Err(e) => Some(Err(e)),
        })
    }

    // // transformations

    // type Map<Q, H>
    //     = XapResBinBin<M, E, X1, MapOf<X2, Q, H>>
    // where
    //     H: Fn(Self::O) -> Q + Copy + Send;

    // fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    // where
    //     H: Fn(Self::O) -> Q + Copy + Send,
    // {
    //     XapResBinBin::new(self.x1, self.x2.map(h))
    // }

    // type Inspect<H>
    //     = XapResBinBin<M, E, X1, X2::Inspect<H>>
    // where
    //     H: Fn(&Self::O) + Copy + Send;

    // fn inspect<H>(self, h: H) -> Self::Inspect<H>
    // where
    //     H: Fn(&Self::O) + Copy + Send,
    // {
    //     XapResBinBin::new(self.x1, self.x2.inspect(h))
    // }

    // type Filter<H>
    //     = XapResBinBin<M, E, X1, X2::Filter<H>>
    // where
    //     H: Fn(&Self::O) -> bool + Copy + Send;

    // fn filter<H>(self, h: H) -> Self::Filter<H>
    // where
    //     H: Fn(&Self::O) -> bool + Copy + Send,
    // {
    //     XapResBinBin::new(self.x1, self.x2.filter(h))
    // }

    // type FilterMap<Q, H>
    //     = XapResBinBin<M, E, X1, X2::FilterMap<Q, H>>
    // where
    //     H: Fn(Self::O) -> Option<Q> + Copy + Send;

    // fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    // where
    //     H: Fn(Self::O) -> Option<Q> + Copy + Send,
    // {
    //     XapResBinBin::new(self.x1, self.x2.filter_map(h))
    // }

    // type FlatMap<V, H>
    //     = XapResBinMany<M, E, X1, X2::FlatMap<V, H>>
    // where
    //     V: IntoIterator,
    //     H: Fn(Self::O) -> V + Copy + Send;

    // fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    // where
    //     V: IntoIterator,
    //     H: Fn(Self::O) -> V + Copy + Send,
    // {
    //     XapResBinMany::new(self.x1, self.x2.flat_map(h))
    // }

    // // transformations - helper

    // type Mapped<H>
    //     = XapResBinBin<M, E, X1, X2::Mapped<H>>
    // where
    //     H: Map<I = Self::O>;

    // fn mapped<H>(self, h: H) -> Self::Mapped<H>
    // where
    //     H: Map<I = Self::O>,
    // {
    //     XapResBinBin::new(self.x1, self.x2.mapped(h))
    // }
}
