use crate::infallible::size::Bin;
use crate::infallible::xap::Xap;
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
        let a = self.x1.xap(i).into_iter().next();
        a.and_then(|a| match a {
            Ok(a) => self.x2.xap(a).into_iter().next().map(Ok),
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
}
