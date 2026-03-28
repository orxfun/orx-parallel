use crate::infallible::Xap;

pub struct XapRes<O, E, X: Xap<O = Result<O, E>>>(X);
