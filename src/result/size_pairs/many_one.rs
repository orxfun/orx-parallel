use crate::infallible::sizes::{Many, One};
use crate::result::size_pairs::SizePair;

pub struct ManyOne;

impl SizePair for ManyOne {
    type S1 = Many;

    type S2 = One;
}
