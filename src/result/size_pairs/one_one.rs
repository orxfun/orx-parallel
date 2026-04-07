use crate::infallible::sizes::One;
use crate::result::size_pairs::SizePair;

pub struct OneOne;

impl SizePair for OneOne {
    type S1 = One;

    type S2 = One;
}
