use crate::infallible::sizes::{Many, One};
use crate::result::size_pairs::SizePair;

pub struct OneMany;

impl SizePair for OneMany {
    type S1 = One;

    type S2 = Many;
}
