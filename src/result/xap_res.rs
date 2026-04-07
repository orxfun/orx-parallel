use crate::infallible::{MapOf, Xap};
use crate::result::size_pairs::SizePair;
use core::marker::PhantomData;

pub struct XapRes<M, E, X1, X2, S>
where
    X1: Xap<O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    x1: X1,
    x2: X2,
    s: PhantomData<S>,
}

impl<M, E, X1, X2, S> XapRes<M, E, X1, X2, S>
where
    X1: Xap<O = Result<M, E>>,
    X2: Xap<I = M>,
    S: SizePair<S1 = X1::Size, S2 = X2::Size>,
{
    pub fn new(x1: X1, x2: X2) -> Self {
        let s = PhantomData;
        Self { x1, x2, s }
    }

    #[inline(always)]
    fn xap_res(&self, i: X1::I) -> S::Results<M, E, X1, X2> {
        S::xap_res(self.x1, self.x2, i)
    }

    // transformations

    fn map<Q, H>(self, h: H) -> XapRes<M, E, X1, MapOf<X2, Q, H>, S>
    where
        H: Fn(X2::O) -> Q + Copy + Send,
    {
        XapRes::new(self.x1, self.x2.map(h))
    }
}
