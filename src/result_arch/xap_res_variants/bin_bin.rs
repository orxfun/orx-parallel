use crate::infallible::{Xap, ZeroOne};
use crate::result::xap_res::{ResOf, XapRes};

pub struct XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = ZeroOne>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = ZeroOne>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResBinBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = ZeroOne>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = Option<ResOf<Self>>;

    #[inline(always)]
    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Self::Results {
        let a = self.x1.xap(i).into_iter().next();
        a.and_then(|a| match a {
            Ok(a) => self.x2.xap(a).into_iter().next().map(Ok),
            Err(e) => Some(Err(e)),
        })
    }
}
