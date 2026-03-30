use crate::infallible::{One, Xap};
use crate::result::xap_res::XapRes;

pub struct XapResOneOne<M, E, X1: Xap<O = Result<M, E>, Count = One>, X2: Xap<I = M, Count = One>> {
    x1: X1,
    x2: X2,
}

impl<M, E, X1: Xap<O = Result<M, E>, Count = One>, X2: Xap<I = M, Count = One>>
    XapResOneOne<M, E, X1, X2>
{
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }
}

impl<M, E, X1: Xap<O = Result<M, E>, Count = One>, X2: Xap<I = M, Count = One>> XapRes
    for XapResOneOne<M, E, X1, X2>
{
    type M = M;

    type E = E;

    type X1 = X1;

    type X2 = X2;

    type Values = [<X2 as Xap>::O; 1];

    #[inline(always)]
    fn xap_res(&self, i: <Self::X1 as Xap>::I) -> Result<Self::Values, Self::E> {
        let a = unsafe { self.x1.xap(i).into_iter().next().unwrap_unchecked() };
        a.map(|a| [unsafe { self.x2.xap(a).into_iter().next().unwrap_unchecked() }])
    }
}
