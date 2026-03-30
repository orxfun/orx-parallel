use crate::infallible::size::One;
use crate::infallible::xap::{Xap, XapOne};
use crate::result::xap_res::{ResOf, XapRes};

pub struct XapResOneOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Size = One>,
    X2: Xap<I = M, Size = One>,
{
    x1: X1,
    x2: X2,
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
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = [ResOf<Self>; 1];

    #[inline(always)]
    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Self::Results {
        let a = self.x1.one_value(i);
        [a.map(|a| self.x2.one_value(a))]
    }

    // transformations

    type Map<Q, H>
        = XapResOneOne<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;

    fn map<Q, H>(self, h: H) -> Self::Map<Q, H>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send,
    {
        XapResOneOne::new(self.x1, self.x2.map(h))
    }

    type Inspect<H>
        = XapResOneOne<M, E, X1, X2::Inspect<H>>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send;

    fn inspect<H>(self, h: H) -> Self::Inspect<H>
    where
        H: Fn(&<Self::X2 as Xap>::O) + Copy + Send,
    {
        XapResOneOne::new(self.x1, self.x2.inspect(h))
    }
}
