use crate::infallible::{MapOf, Xap};

pub struct XapRes<M, E, X1: Xap<O = Result<M, E>>, X2: Xap<I = M>> {
    x1: X1,
    x2: X2,
}

impl<M, E, X1: Xap<O = Result<M, E>>, X2: Xap<I = M>> XapRes<M, E, X1, X2> {
    pub fn new(x1: X1, x2: X2) -> Self {
        Self { x1, x2 }
    }

    // transformations

    fn map<Q, H>(self, h: H) -> XapRes<M, E, X1, MapOf<X2, Q, H>>
    where
        H: Fn(X2::O) -> Q + Copy + Send,
    {
        XapRes::new(self.x1, self.x2.map(h))
    }
}
