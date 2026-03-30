use crate::infallible::{One, Xap, ZeroOne};
use crate::result::xap_res::{ResOf, XapRes};

pub struct XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = One>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = One>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResBinOne<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = One>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = Option<ResOf<Self>>;

    #[inline(always)]
    fn xap_res(&self, i: Self::I) -> Self::Results {
        let a = self.x1.xap(i).into_iter().next();
        a.map(|a| a.map(|a| unsafe { self.x2.xap(a).into_iter().next().unwrap_unchecked() }))
    }
}
