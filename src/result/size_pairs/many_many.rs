use crate::infallible::sizes::Many;
use crate::result::size_pairs::SizePair;

pub struct ManyMany;

impl SizePair for ManyMany {
    type S1 = Many;

    type S2 = Many;
}
