use crate::infallible::{Many, Xap, ZeroOne};
use crate::result::xap_res::XapRes;
use core::iter::Flatten;
use core::option::IntoIter;

pub struct XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = Many>,
{
    x1: X1,
    x2: X2,
}

impl<M, E, X1, X2> XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = Many>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1, X2> XapRes for XapResBinMany<M, E, X1, X2>
where
    X1: Xap<O = Result<M, E>, Count = ZeroOne>,
    X2: Xap<I = M, Count = Many>,
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Values = Flatten<IntoIter<<X2 as Xap>::Values>>;

    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Result<Self::Values, Self::E> {
        let res = match self.x1.xap(i).into_iter().next() {
            Some(Ok(a)) => Ok(Some(self.x2.xap(a))),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        };
        res.map(|z| z.into_iter().flatten())
    }
}
