use crate::infallible::sizes::Size;

pub trait SizePair {
    type S1: Size;

    type S2: Size;
}
