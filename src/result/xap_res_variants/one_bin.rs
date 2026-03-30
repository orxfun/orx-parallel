use crate::infallible::{One, Xap, ZeroOne};
use crate::result::xap_res::{ResOf, XapRes};

pub struct XapResOneBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = One>,
    X2: Xap<I = M>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResOneBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = One>,
    X2: Xap<I = M, Count = ZeroOne>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResOneBin<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = One>,
    X2: Xap<I = M>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Results = Option<ResOf<Self>>;

    #[inline(always)]
    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Self::Results {
        // SAFETY: X1::Count = One by the trait bound
        match unsafe { self.x1.xap(i).into_iter().next().unwrap_unchecked() } {
            Ok(a) => self.x2.xap(a).into_iter().next().map(Ok),
            Err(e) => Some(Err(e)),
        }
    }

    // transformations

    type Map<Q, H>
        = XapResOneBin<M, E, X1, X2::Map<Q, H>>
    where
        H: Fn(<Self::X2 as Xap>::O) -> Q + Copy + Send;
}
