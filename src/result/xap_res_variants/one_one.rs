use crate::infallible::{One, Xap};

pub struct XapResOneOne<M, E, X1: Xap<O = Result<M, E>, Count = One>, X2: Xap<I = M, Count = One>> {
    x1: X1,
    x2: X2,
}
