use crate::infallible::fun::Map;
use crate::infallible::sizes::One;
use crate::infallible::{Xap, XapOne};
use crate::result::xap_res::{ResOf, XapRes};
use crate::result::xap_res_variants::{XapResOneBin, XapResOneMany};

pub struct XapResOneOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = One>,
    X2: Xap<I = M, Size = One>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> Clone for XapResOneOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = One>,
    X2: Xap<I = M, Size = One>,
{
    fn clone(&self) -> Self {
        let (x1, x2) = (self.x1, self.x2);
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> Copy for XapResOneOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = One>,
    X2: Xap<I = M, Size = One>,
{
}

unsafe impl<M, E, X1, X2> Send for XapResOneOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = One>,
    X2: Xap<I = M, Size = One>,
{
}

impl<M, E, X1, X2> XapResOneOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = One>,
    X2: Xap<I = M, Size = One>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResOneOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = One>,
    X2: Xap<I = M, Size = One>,
{
    type I = X1::I;

    type M = M;

    type E = E;

    type O = X2::O;

    type Results = [ResOf<Self>; 1];

    #[inline(always)]
    fn xap_res(&self, i: Self::I) -> Self::Results {
        let a = self.x1.one_value(i);
        [a.map(|a| self.x2.one_value(a))]
    }

    // transformations

    type Map<Q, H>
        = XapResOneOne<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(Self::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(Self::O) -> Q + Copy + Send,
    {
        XapResOneOne::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapResOneOne<M, E, X1, X2::Inspect<H>>
    where
        H: Fn(&Self::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&Self::O) + Copy + Send,
    {
        XapResOneOne::new(self.x1, self.x2.inspect(h))
    }

    type Filter<H>
        = XapResOneBin<M, E, X1, X2::Filter<H>>
    where
        H: Fn(&Self::O) -> bool + Copy + Send;

    fn filter<H>(self, h: H) -> Self::Filter<H>
    where
        H: Fn(&Self::O) -> bool + Copy + Send,
    {
        XapResOneBin::new(self.x1, self.x2.filter(h))
    }

    type FilterMap<Q, H>
        = XapResOneBin<M, E, X1, X2::FilterMap<Q, H>>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send;

    fn filter_map<Q, H>(self, h: H) -> Self::FilterMap<Q, H>
    where
        H: Fn(Self::O) -> Option<Q> + Copy + Send,
    {
        XapResOneBin::new(self.x1, self.x2.filter_map(h))
    }

    type FlatMap<V, H>
        = XapResOneMany<M, E, X1, X2::FlatMap<V, H>>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send;

    fn flat_map<V, H>(self, h: H) -> Self::FlatMap<V, H>
    where
        V: IntoIterator,
        H: Fn(Self::O) -> V + Copy + Send,
    {
        XapResOneMany::new(self.x1, self.x2.flat_map(h))
    }

    // transformations - helper

    type Mapped<H>
        = XapResOneOne<M, E, X1, X2::Mapped<H>>
    where
        H: Map<I = Self::O>;

    fn mapped<H>(self, h: H) -> Self::Mapped<H>
    where
        H: Map<I = Self::O>,
    {
        XapResOneOne::new(self.x1, self.x2.mapped(h))
    }
}
