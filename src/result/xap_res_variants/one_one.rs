use crate::infallible::size::One;
use crate::infallible::xap::Xap;
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
        // SAFETY: X1::Size= One by the trait bound
        let a = unsafe { self.x1.xap(i).into_iter().next().unwrap_unchecked() };

        // SAFETY: X2::Size= One is satisfied by the only public constructor
        [a.map(|a| unsafe { self.x2.xap(a).into_iter().next().unwrap_unchecked() })]
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
}
