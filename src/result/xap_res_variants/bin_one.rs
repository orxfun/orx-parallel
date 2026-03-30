use crate::infallible::{One, Xap, ZeroOne};
use crate::result::xap_res::XapRes;

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

    type Values = Option<<X2 as Xap>::O>;

    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Result<Self::Values, Self::E> {
        match self.x1.xap(i).into_iter().next() {
            Some(a) => {
                match a.map(|a| unsafe { self.x2.xap(a).into_iter().next().unwrap_unchecked() }) {
                    Ok(b) => Ok(Some(b)),
                    Err(e) => Err(e),
                }
            }
            None => Ok(None),
        }
    }
}
